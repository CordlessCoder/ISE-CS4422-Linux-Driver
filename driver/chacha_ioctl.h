#ifndef CHACHA_IOCTL_H_
#define CHACHA_IOCTL_H_

#include "asm-generic/ioctl.h"
#include <linux/fs.h>

#define SET_KEY _IOW('s', 'k', char[32])
#define SET_NONCE _IOW('s', 'n', char[8])
#define RESET_COUNTER _IO('r', 'c')
#define SET_COUNTER _IOW('s', 'c', u64)
#define CLEAR_OUTPUT_ONLY _IO('c', 'o')
#define SET_OUTPUT_ONLY _IO('s', 'o')

long int chacha_ioctl(struct file* f, unsigned int cmd, unsigned long args);

#endif
