#ifndef CHACHA_H
#define CHACHA_H

#include "../chacha20/chacha20.h"
#include "linux/dev_printk.h"
#include "linux/dynamic_debug.h"
#include "linux/mutex.h"
#include <linux/cdev.h>
#include <linux/fs.h>
#include <linux/init.h>
#include <linux/module.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/atomic.h>

extern dev_t lchacha_dev_number;
extern struct cdev lchacha_cdev;
extern const struct device* lchacha_dev;
extern const struct class* lchacha_device_class;
extern struct proc_dir_entry* lchacha_proc_file;

#define PROC_FILENAME "chastats"

#define STATE_SIZE 4096

#define STATE_FIELDS                                                                                                                                                                                   \
    ChaCha20Ctx ctx;                                                                                                                                                                                   \
    struct mutex lock;                                                                                                                                                                                 \
    /* Circular buffer for data to be processed to be available for reading*/                                                                                                                          \
    u64 offset;                                                                                                                                                                                        \
    u16 len;                                                                                                                                                                                           \
    /* Allows the user to request the cipher directly, without XORing with input */                                                                                                                    \
    u64 requested_zeroed_inputs;

#define BUF_CAPACITY (STATE_SIZE - sizeof(struct {STATE_FIELDS}))

// stats

struct chacha_stats{
    atomic_t reads;
    atomic_t writes;
    atomic_t blocks;
    atomic_t errors;

    atomic_t ioctls;
    atomic_t current_buffer_bytes;

    atomic_t total_sessions;
    atomic_t active_sessions;
    atomic_t bytes_processed;
};

extern struct chacha_stats lchacha_stats;

typedef struct {
    STATE_FIELDS
    u8 buffer[BUF_CAPACITY];
} chacha_state;

ssize_t lchacha_read(struct file* f, char __user* user_buf, size_t len, loff_t* offset);
ssize_t lchacha_write(struct file* f, const char __user* user_buf, size_t len, loff_t* offset);
ssize_t lchacha_proc_read(struct file* file, char __user* user_buffer, size_t count, loff_t* position);
loff_t lchacha_lseek(struct file* f, loff_t offset, int whence);
int lchacha_proc_open(struct inode* inode, struct file* file);

#endif
