#include "chacha.h"
#include "chacha_ioctl.h"
#include "linux/types.h"

dev_t lchacha_dev_number;
struct cdev lchacha_cdev;
const struct device* lchacha_dev;
const struct class* lchacha_device_class;
struct proc_dir_entry* lchacha_proc_file;

atomic64_t lchacha_total_sessions = ATOMIC_INIT(0);
atomic64_t lchacha_active_sessions = ATOMIC_INIT(0);
atomic64_t lchacha_bytes_processed = ATOMIC_INIT(0);

static int chacha_open(struct inode* _, struct file* f) {
    dev_info(lchacha_dev, "Open is called\n");
    f->private_data = kcalloc(STATE_SIZE, 1, GFP_KERNEL_ACCOUNT);
    if (!f->private_data) {
        pr_err("chacha - Out of memory\n");
        return -ENOMEM;
    }
    char zero_key[32] = {};
    chacha_state* state = f->private_data;

    ChaCha20_set_key(&state->ctx, zero_key);
    mutex_init(&state->lock);

    atomic64_inc(&lchacha_total_sessions);
    atomic64_inc(&lchacha_active_sessions);

    return 0;
}

static int chacha_release(struct inode* _, struct file* f) {
    dev_info(lchacha_dev, "Release is called\n");
    {
        chacha_state* state = f->private_data;
        mutex_destroy(&state->lock);
    }
    kfree(f->private_data);
    atomic64_dec(&lchacha_active_sessions);
    return 0;
}

static struct file_operations fops = {.read = lchacha_read, .open = chacha_open, .write = lchacha_write, .release = chacha_release, .unlocked_ioctl = chacha_ioctl};
// File operations
static const struct proc_ops proc_fops = {
    .proc_open = lchacha_proc_open,
    .proc_read = seq_read,
    .proc_lseek = seq_lseek,
    .proc_release = single_release,
};


static int __init dev_init(void) {
    int status;
#ifdef STATIC_DEVNR
    dev_nr = STATIC_DEVNR;
    status = register_chrdev_region(dev_nr, MINORMASK + 1, "chacha");
#else
    status = alloc_chrdev_region(&lchacha_dev_number, 0, MINORMASK + 1, "chacha");
#endif
    if (status) {
        pr_err("chacha - Error reserving the region of device numbers\n");
        return status;
    }

    cdev_init(&lchacha_cdev, &fops);
    lchacha_cdev.owner = THIS_MODULE;

    status = cdev_add(&lchacha_cdev, lchacha_dev_number, MINORMASK + 1);
    if (status) {
        pr_err("chacha - Error adding cdev\n");
        goto free_devnr;
    }

    pr_info("chacha - Registered a character device for Major %d starting with Minor %d\n", MAJOR(lchacha_dev_number), MINOR(lchacha_dev_number));

    lchacha_device_class = class_create("crypto");
    if (!lchacha_device_class) {
        pr_err("chacha - Could not create class my_class\n");
        status = ENOMEM;
        goto delete_cdev;
    }

    if (!(lchacha_dev = device_create(lchacha_device_class, NULL, lchacha_dev_number, NULL, "chacha"))) {
        pr_err("chacha - Could not create device chacha\n");
        status = ENOMEM;
        goto delete_class;
    }

    dev_info(lchacha_dev, "Created device under /sys/class/my_class/chacha\n");

    lchacha_proc_file = proc_create(PROC_FILENAME, 0666, NULL, &proc_fops);
    if (!lchacha_proc_file) {
        status = ENOMEM;
        goto delete_device;
    }

    dev_info(lchacha_dev, "Created entry under /proc/%s\n", PROC_FILENAME);

    return 0;

delete_device:
    device_destroy(lchacha_device_class, lchacha_dev_number);
delete_class:
    class_unregister(lchacha_device_class);
    class_destroy(lchacha_device_class);
delete_cdev:
    cdev_del(&lchacha_cdev);
free_devnr:
    unregister_chrdev_region(lchacha_dev_number, MINORMASK + 1);
    return status;
}

static void __exit dev_exit(void) {
    remove_proc_entry(PROC_FILENAME, NULL);
    device_destroy(lchacha_device_class, lchacha_dev_number);
    class_unregister(lchacha_device_class);
    class_destroy(lchacha_device_class);
    cdev_del(&lchacha_cdev);
    unregister_chrdev_region(lchacha_dev_number, MINORMASK + 1);
}

module_init(dev_init);
module_exit(dev_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("CordlessCoder");
MODULE_DESCRIPTION("A simple ChaCha20 encryption implementation as a kernel driver");
