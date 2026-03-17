#include <linux/cdev.h>
#include <linux/fs.h>
#include <linux/init.h>
#include <linux/module.h>

#include "../chacha20/chacha20.c"
#include "linux/dev_printk.h"
#include "linux/dynamic_debug.h"
#include "linux/errno.h"
#include "linux/mutex.h"
#include "linux/printk.h"
#include "linux/uaccess.h"
#include "linux/wait_bit.h"

static dev_t dev_number;
static struct cdev cdev_instance;
static const struct device* dev_instance;
static const struct class* device_class;


#define STATE_SIZE 4096


#define STATE_FIELDS                                                                                                                                                                                   \
    ChaCha20Ctx ctx;                                                                                                                                                                                   \
    /* Offset within the current 64-byte ChaCha20 block */                                                                                                                                             \
    u8 chacha_block_offset;                                                                                                                                                                            \
    struct mutex lock;                                                                                                                                                                                 \
    /* Circular buffer for data to be processed to be available for reading*/                                                                                                                          \
    u16 len;                                                                                                                                                                                           \
    u16 start;

#define BUF_CAPACITY (STATE_SIZE - sizeof(struct {STATE_FIELDS}))

typedef struct {
    STATE_FIELDS
    u8 buffer[BUF_CAPACITY];
} chacha_state;

void chacha_process(chacha_state* state, char* data, size_t len);

static int chacha_open(struct inode* _, struct file* f) {
    dev_info(dev_instance, "Open is called\n");
    f->private_data = kmalloc(STATE_SIZE, GFP_KERNEL_ACCOUNT);
    if (!f->private_data) {
        pr_err("chacha - Out of memory\n");
        return -ENOMEM;
    }
    char zero_key[32] = {};
    chacha_state* state = f->private_data;

    memset(state, 0, STATE_SIZE);
    ChaCha20_set_key(&state->ctx, zero_key);
    mutex_init(&state->lock);

    return 0;
}

static int chacha_release(struct inode* _, struct file* f) {
    dev_info(dev_instance, "Release is called\n");

    {
        chacha_state* state = f->private_data;
        mutex_destroy(&state->lock);
    }

    kfree(f->private_data);
    return 0;
}

void chacha_process(chacha_state* state, char* data, size_t len) {
    char block[CHACHA20_BLOCKLENGTH] = {};
    while (len != 0) {
        if (state->chacha_block_offset == 0 && len >= CHACHA20_BLOCKLENGTH) {
            // We can use a xorblock operation with no padding
            ChaCha20_xorblock_noinc(&state->ctx, data);
            ChaCha20_increment_counter(&state->ctx);

            data += CHACHA20_BLOCKLENGTH;
            len -= CHACHA20_BLOCKLENGTH;
            continue;
        }
        // We have a partial block, or an existing offset
        size_t start = state->chacha_block_offset;
        size_t chunk_len = min(len, CHACHA20_BLOCKLENGTH - start);

        memcpy(&block[start], data, chunk_len);
        ChaCha20_xorblock_noinc(&state->ctx, block);
        memcpy(data, &block[start], chunk_len);

        data += chunk_len;
        len -= chunk_len;
        state->chacha_block_offset = (state->chacha_block_offset + chunk_len) % CHACHA20_BLOCKLENGTH;
        if (state->chacha_block_offset == 0) {
            ChaCha20_increment_counter(&state->ctx);
        }
    }
}

static ssize_t chacha_read(struct file* f, char __user* user_buf, size_t len, loff_t* offset) {
    if (!len) {
        return 0;
    }
    dev_info(dev_instance, "read(%zu) called", len);
    chacha_state* state = f->private_data;
    if (mutex_lock_interruptible(&state->lock)) {
        return -ERESTARTSYS;
    }
    // Wait for buffer to become non-empty
    if (wait_var_event_any_lock(&state->len, state->len != 0, &state->lock, mutex, TASK_INTERRUPTIBLE)) {
        return -ERESTARTSYS;
    };

    // first try to copy the part of the ring buffer that's before the wrap
    size_t output = 0;
    for (;;) {
        size_t available_before_wrap = min(state->len, BUF_CAPACITY - state->start);
        size_t can_read = min(available_before_wrap, len);
        if (can_read == 0) {
            break;
        }
        chacha_process(state, &state->buffer[state->start], can_read);
        if (copy_to_user(user_buf, &state->buffer[state->start], can_read)) {
            dev_err(dev_instance, "Failed to copy input to write() to user\n");
            return -EFAULT;
        };
        dev_info_ratelimited(dev_instance, "Copied %zu bytes to user", can_read);
        user_buf += can_read;
        len -= can_read;
        output += can_read;

        state->start = (state->start + can_read) % BUF_CAPACITY;
        state->len -= can_read;
    }

    wake_up_var_locked(&state->start, &state->lock);
    mutex_unlock(&state->lock);
    return output;
}

static ssize_t chacha_write(struct file* f, const char __user* user_buf, size_t len, loff_t* offset) {
    if (!len) {
        return 0;
    }
    chacha_state* state = f->private_data;
    if (mutex_lock_interruptible(&state->lock)) {
        return -ERESTARTSYS;
    }
    dev_info(dev_instance, "write(%zu) called", len);

    // Wait for buffer to become non-full
    if (wait_var_event_any_lock(&state->start, state->len != BUF_CAPACITY, &state->lock, mutex, TASK_INTERRUPTIBLE)) {
        return -ERESTARTSYS;
    };
    size_t copied = 0;
    while (len) {
        size_t remaining_space_in_buffer = BUF_CAPACITY - state->len;
        if (remaining_space_in_buffer == 0) {
            break;
        }
        size_t writable_start = (state->start + state->len) % BUF_CAPACITY;
        size_t writable_len = BUF_CAPACITY - writable_start;
        if (writable_start < state->start) {
            writable_len = state->start - writable_start;
        }
        size_t to_copy = min(len, writable_len);
        if (copy_from_user(&state->buffer[writable_start], user_buf, to_copy)) {
            dev_err(dev_instance, "Failed to copy input to write() from user\n");
            return -EFAULT;
        };
        dev_info_ratelimited(dev_instance, "Copied %zu bytes from user", to_copy);
        user_buf += to_copy;
        len -= to_copy;
        copied += to_copy;
        state->len += to_copy;
    }
    wake_up_var_locked(&state->len, &state->lock);
    mutex_unlock(&state->lock);
    return copied;
}


static struct file_operations fops = {.read = chacha_read, .open = chacha_open, .write = chacha_write, .release = chacha_release};

static int __init dev_init(void) {
    int status;
#ifdef STATIC_DEVNR
    dev_nr = STATIC_DEVNR;
    status = register_chrdev_region(dev_nr, MINORMASK + 1, "chacha");
#else
    status = alloc_chrdev_region(&dev_number, 0, MINORMASK + 1, "chacha");
#endif
    if (status) {
        pr_err("chacha - Error reserving the region of device numbers\n");
        return status;
    }

    cdev_init(&cdev_instance, &fops);
    cdev_instance.owner = THIS_MODULE;

    status = cdev_add(&cdev_instance, dev_number, MINORMASK + 1);
    if (status) {
        pr_err("chacha - Error adding cdev\n");
        goto free_devnr;
    }

    pr_info("chacha - Registered a character device for Major %d starting with Minor %d\n", MAJOR(dev_number), MINOR(dev_number));

    device_class = class_create("crypto");
    if (!device_class) {
        pr_err("chacha - Could not create class my_class\n");
        status = ENOMEM;
        goto delete_cdev;
    }

    if (!(dev_instance = device_create(device_class, NULL, dev_number, NULL, "chacha"))) {
        pr_err("chacha - Could not create device chacha\n");
        status = ENOMEM;
        goto delete_class;
    }

    dev_info(dev_instance, "Created device under /sys/class/my_class/chacha\n");

    return 0;

delete_class:
    class_unregister(device_class);
    class_destroy(device_class);
delete_cdev:
    cdev_del(&cdev_instance);
free_devnr:
    unregister_chrdev_region(dev_number, MINORMASK + 1);
    return status;
}

static void __exit dev_exit(void) {
    device_destroy(device_class, dev_number);
    class_unregister(device_class);
    class_destroy(device_class);
    cdev_del(&cdev_instance);
    unregister_chrdev_region(dev_number, MINORMASK + 1);
}

module_init(dev_init);
module_exit(dev_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("CordlessCoder");
MODULE_DESCRIPTION("A simple ChaCha20 encryption implementation as a kernel driver");
