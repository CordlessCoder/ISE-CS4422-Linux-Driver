#include "chacha.h"
#include "chacha_ioctl.h"

long int chacha_ioctl(struct file* f, unsigned int cmd, unsigned long args) {
    chacha_state* state = f->private_data;
    dev_dbg(lchacha_dev, "ioctl called with cmd: 0x%x and args: %p\n", cmd, (void*)args);
    int status = 0;

    switch (cmd) {
    case SET_KEY: {
        char key[32];
        status = copy_from_user(key, (typeof(key)*)args, sizeof(key));
        if (status) {
            dev_err(lchacha_dev, "Error on SET_KEY\n");
            return status;
        }
        ChaCha20_set_key(&state->ctx, key);
    } break;
    case SET_NONCE: {
        char nonce[8];
        status = copy_from_user(nonce, (typeof(nonce)*)args, sizeof(nonce));
        if (status) {
            dev_err(lchacha_dev, "Error on SET_NONCE\n");
            return status;
        }
        ChaCha20_set_nonce(&state->ctx, nonce);
    } break;
    case RESET_COUNTER: {
        ChaCha20_set_counter(&state->ctx, 0);
        state->chacha_block_offset = 0;
    } break;
    case SET_COUNTER: {
        u64 counter;
        status = copy_from_user(&counter, (typeof(counter)*)args, sizeof(counter));
        if (status) {
            dev_err(lchacha_dev, "Error on SET_COUNTER\n");
            return status;
        }
        state->chacha_block_offset = 0;
        ChaCha20_set_counter(&state->ctx, counter);
    } break;

    default: {
        return -EOPNOTSUPP;
    } break;
    }
    return status;
}
