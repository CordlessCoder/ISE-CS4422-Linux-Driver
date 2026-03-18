#include "chacha.h"
#include "linux/overflow.h"
#include "linux/wait.h"

static void chacha_process(chacha_state* state, char* data, size_t len);

ssize_t lchacha_read(struct file* f, char __user* user_buf, size_t len, loff_t* offset) {
    if (!len) {
        return 0;
    }
    dev_dbg(lchacha_dev, "read(%zu) called", len);
    chacha_state* state = f->private_data;
    if (mutex_lock_interruptible(&state->lock)) {
        return -ERESTARTSYS;
    }
    // Wait for buffer to become non-empty
    int status;
    if ((status = wait_var_event_any_lock(&state->len, state->len != 0, &state->lock, mutex, TASK_INTERRUPTIBLE))) {
        return status;
    };

    // first try to copy the part of the ring buffer that's before the wrap
    size_t output = 0;
    for (;;) {
        size_t start_in_buf = state->offset % BUF_CAPACITY;
        size_t available_before_wrap = min(state->len, BUF_CAPACITY - start_in_buf);
        size_t can_read = min(available_before_wrap, len);
        dev_dbg(lchacha_dev, "Read iteration with %zu bytes available in the buffer", can_read);
        if (can_read == 0) {
            break;
        }
        dev_dbg(lchacha_dev, "Reading %zu bytes starting at %zu", can_read, start_in_buf);
        chacha_process(state, &state->buffer[start_in_buf], can_read);
        state->len -= can_read;
        if (copy_to_user(user_buf, &state->buffer[start_in_buf], can_read)) {
            dev_err(lchacha_dev, "Failed to copy input to write() to user\n");
            output = -EFAULT;
            break;
        };
        dev_dbg(lchacha_dev, "Copied %zu bytes to user", can_read);
        user_buf += can_read;
        len -= can_read;
        output += can_read;
    }

    dev_dbg(lchacha_dev, "Read iteration result: start = %llu, len = %hu", state->offset % BUF_CAPACITY, state->len);
    wake_up_var_locked(&state->len, &state->lock);
    mutex_unlock(&state->lock);
    return output;
}

ssize_t lchacha_write(struct file* f, const char __user* user_buf, size_t len, loff_t* offset) {
    if (!len) {
        return 0;
    }
    chacha_state* state = f->private_data;
    if (mutex_lock_interruptible(&state->lock)) {
        return -ERESTARTSYS;
    }
    dev_dbg(lchacha_dev, "write(%zu) called", len);

    // Wait for buffer to become non-full
    int status = 0;
    if ((status = wait_var_event_any_lock(&state->len, state->len != BUF_CAPACITY, &state->lock, mutex, TASK_INTERRUPTIBLE))) {
        return status;
    };
    size_t output = 0;
    while (len) {
        size_t remaining_space_in_buffer = BUF_CAPACITY - state->len;
        dev_dbg(lchacha_dev, "Write iteration with %zu bytes left in the buffer", remaining_space_in_buffer);
        if (remaining_space_in_buffer == 0) {
            break;
        }
        size_t start_in_buf = state->offset % BUF_CAPACITY;
        dev_dbg(lchacha_dev, "write found: start = %zu, len = %hu", start_in_buf, state->len);
        size_t writable_start = (start_in_buf + state->len) % BUF_CAPACITY;
        size_t writable_len = BUF_CAPACITY - writable_start;
        if (writable_start < start_in_buf) {
            writable_len = start_in_buf - writable_start;
        }
        size_t to_copy = min(len, writable_len);
        dev_dbg(lchacha_dev, "Writing %zu bytes starting at %zu", to_copy, writable_start);
        if (copy_from_user(&state->buffer[writable_start], user_buf, to_copy)) {
            dev_err(lchacha_dev, "Failed to copy input to write() from user\n");
            output = -EFAULT;
            break;
        };
        dev_dbg(lchacha_dev, "Copied %zu bytes from user", to_copy);
        user_buf += to_copy;
        len -= to_copy;
        output += to_copy;
        state->len += to_copy;
    }
    wake_up_var_locked(&state->len, &state->lock);
    mutex_unlock(&state->lock);
    return output;
}

// NOTE: This function will advance the offset by the amount it reads, do not do that outside of it!
static void chacha_process(chacha_state* state, char* data, size_t len) {
    dev_dbg(lchacha_dev, "Processing ChaCha20, len = %zu, offset = %llu", len, state->offset);
    atomic64_add(len, &lchacha_bytes_processed);

    char block[CHACHA20_BLOCKLENGTH] = {};
    while (len != 0) {
        if ((state->offset % CHACHA20_BLOCKLENGTH) == 0 && len >= CHACHA20_BLOCKLENGTH) {
            // We can use a xorblock operation with no padding
            ChaCha20_xorblock_noinc(&state->ctx, data);
            ChaCha20_increment_counter(&state->ctx);

            data += CHACHA20_BLOCKLENGTH;
            len -= CHACHA20_BLOCKLENGTH;
            state->offset += CHACHA20_BLOCKLENGTH;

            continue;
        }
        // We have a partial block, or an existing offset
        size_t start = state->offset % CHACHA20_BLOCKLENGTH;
        size_t chunk_len = min(len, CHACHA20_BLOCKLENGTH - start);

        memcpy(&block[start], data, chunk_len);
        ChaCha20_xorblock_noinc(&state->ctx, block);
        memcpy(data, &block[start], chunk_len);

        data += chunk_len;
        len -= chunk_len;
        state->offset += chunk_len;
        if ((state->offset % CHACHA20_BLOCKLENGTH) == 0) {
            ChaCha20_increment_counter(&state->ctx);
        }
    }
}

loff_t lchacha_lseek(struct file* f, loff_t offset, int whence) {
    chacha_state* state = f->private_data;
    dev_dbg(lchacha_dev, "lseek called, current pos = %lld, action = %d, offset = %lld\n", f->f_pos, whence, offset);

    int status = 0;

    if (mutex_lock_interruptible(&state->lock)) {
        return -ERESTARTSYS;
    }

    switch (whence) {
    case SEEK_SET: {
        if (offset < 0) {
            status = -EINVAL;
            goto unlock;
        }
        state->offset = offset;
    } break;
    case SEEK_CUR: {
        s64 temp = (s64)state->offset + (s64)offset;
        if (temp < 0) {
            status = -EINVAL;
            goto unlock;
        }
        state->offset = temp;
    } break;
    case SEEK_END: {
        status = -EOPNOTSUPP;
        goto unlock;
    } break;
    default: {
        status = -EINVAL;
        goto unlock;
    } break;
    }

    state->len = 0;

    dev_dbg(lchacha_dev, "New offset: %lld\n", state->offset);
    ChaCha20_set_counter(&state->ctx, state->offset / CHACHA20_BLOCKLENGTH);

unlock:
    mutex_unlock(&state->lock);
    return status;
}
