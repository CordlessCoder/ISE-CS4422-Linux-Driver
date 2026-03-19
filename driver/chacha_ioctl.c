#include "chacha_ioctl.h"
#include "chacha.h"

long int chacha_ioctl(struct file* f, unsigned int cmd, unsigned long args) {
    atomic_inc(&lchacha_stats.ioctls);
    
    chacha_state* state = f->private_data;
    dev_dbg(lchacha_dev, "ioctl called with cmd: 0x%x and args: %p\n", cmd, (void*)args);
    int status = 0;

    if (mutex_lock_interruptible(&state->lock)) {
        return -ERESTARTSYS;
    }

    switch (cmd) {
    case SET_KEY: {
        char key[32];
        status = copy_from_user(key, (typeof(key)*)args, sizeof(key));
        if (status) {
            dev_err(lchacha_dev, "Error on SET_KEY\n");
            goto unlock;
        }
        ChaCha20_set_key(&state->ctx, key);
    } break;
    case SET_NONCE: {
        char nonce[8];
        status = copy_from_user(nonce, (typeof(nonce)*)args, sizeof(nonce));
        if (status) {
            dev_err(lchacha_dev, "Error on SET_NONCE\n");
            goto unlock;
        }
        ChaCha20_set_nonce(&state->ctx, nonce);
    } break;
    case RESET_COUNTER: {
        ChaCha20_set_counter(&state->ctx, 0);
        state->offset = 0;
    } break;
    case SET_COUNTER: {
        u64 counter;
        status = copy_from_user(&counter, (typeof(counter)*)args, sizeof(counter));
        if (status) {
            dev_err(lchacha_dev, "Error on SET_COUNTER\n");
            goto unlock;
        }
        state->offset = CHACHA20_BLOCKLENGTH * counter;
        ChaCha20_set_counter(&state->ctx, counter);
    } break;
    case CLEAR_ZEROES: {
        state->requested_zeroed_inputs = 0;
    } break;
    case REQUEST_ZEROES: {
        u64 count;
        status = copy_from_user(&count, (typeof(count)*)args, sizeof(count));
        if (status) {
            dev_err(lchacha_dev, "Error on REQUEST_ZEROES\n");
            goto unlock;
        }
        state->requested_zeroed_inputs += count;
    } break;

    default: {
        atomic_inc(&lchacha_stats.errors);
        status = -EOPNOTSUPP;
    } break;
    }
unlock:
    mutex_unlock(&state->lock);
    return status;
}
