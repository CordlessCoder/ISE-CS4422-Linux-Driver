#include <linux/cdev.h>
#include <linux/fs.h>
#include <linux/init.h>
#include <linux/module.h>

#include "../chacha20/chacha20.c"
#include "linux/mutex.h"

static dev_t dev_number;
static struct cdev cdev_instance;
static struct class* device_class;

#define STATE_SIZE 4096
#define BUF_CAPACITY (STATE_SIZE - sizeof(struct chacha_state_without_buffer))


struct chacha_state_without_buffer {
    ChaCha20Ctx ctx;
    struct mutex lock;
    // Circular buffer for data to be processed to be available for reading
    u16 len;
    u16 start;
};

typedef struct {
    struct chacha_state_without_buffer;
    u8 buffer[BUF_CAPACITY];
} chacha_state;

static int chacha_open(struct inode* _, struct file* f) {
    pr_info("chacha - Open is called\n");
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
    pr_info("chacha - Release is called\n");

    {
        chacha_state* state = f->private_data;
        mutex_destroy(&state->lock);
    }

    kfree(f->private_data);
    return 0;
}

static ssize_t chacha_read(struct file* f, char __user* user_buf, size_t len, loff_t* offset) {
    chacha_state* state = f->private_data;
    pr_info("chacha - Read is called\n");
    return 0;
}

static ssize_t chacha_write(struct file* f, const char __user* user_buf, size_t len, loff_t* offset) {
    chacha_state* state = f->private_data;
    pr_info("chacha - Write is called\n");
    return 0;
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

    if (!device_create(device_class, NULL, dev_number, NULL, "hello%d", 0)) {
        pr_err("chacha - Could not create device hello0\n");
        status = ENOMEM;
        goto delete_class;
    }

    pr_info("chacha - Created device under /sys/class/my_class/hello0\n");

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
