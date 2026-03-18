#include "chacha.h"

static void chacha_process(chacha_state* state, char* data, size_t len);

ssize_t lchacha_read(struct file* f, char __user* user_buf, size_t len, loff_t* offset) {
    if (!len) {
        return 0;
    }
    dev_info(lchacha_dev, "read(%zu) called", len);
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
        size_t available_before_wrap = min(state->len, BUF_CAPACITY - *offset);
        size_t can_read = min(available_before_wrap, len);
        if (can_read == 0) {
            break;
        }
        chacha_process(state, &state->buffer[*offset], can_read);
        if (copy_to_user(user_buf, &state->buffer[*offset], can_read)) {
            dev_err(lchacha_dev, "Failed to copy input to write() to user\n");
            return -EFAULT;
        };
        dev_info_ratelimited(lchacha_dev, "Copied %zu bytes to user", can_read);
        user_buf += can_read;
        len -= can_read;
        output += can_read;

        *offset = (*offset + can_read) % BUF_CAPACITY;
        state->len -= can_read;
    }

    wake_up_var_locked(offset, &state->lock);
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
    dev_info(lchacha_dev, "write(%zu) called", len);

    // Wait for buffer to become non-full
    if (wait_var_event_any_lock(&*offset, state->len != BUF_CAPACITY, &state->lock, mutex, TASK_INTERRUPTIBLE)) {
        return -ERESTARTSYS;
    };
    size_t copied = 0;
    while (len) {
        size_t remaining_space_in_buffer = BUF_CAPACITY - state->len;
        if (remaining_space_in_buffer == 0) {
            break;
        }
        size_t writable_start = (*offset + state->len) % BUF_CAPACITY;
        size_t writable_len = BUF_CAPACITY - writable_start;
        if (writable_start < *offset) {
            writable_len = *offset - writable_start;
        }
        size_t to_copy = min(len, writable_len);
        if (copy_from_user(&state->buffer[writable_start], user_buf, to_copy)) {
            dev_err(lchacha_dev, "Failed to copy input to write() from user\n");
            return -EFAULT;
        };
        dev_info_ratelimited(lchacha_dev, "Copied %zu bytes from user", to_copy);
        user_buf += to_copy;
        len -= to_copy;
        copied += to_copy;
        state->len += to_copy;
    }
    wake_up_var_locked(&state->len, &state->lock);
    mutex_unlock(&state->lock);
    return copied;
}

static void chacha_process(chacha_state* state, char* data, size_t len) {
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
        atomic64_add(chunk_len, &lchacha_bytes_processed);
        state->chacha_block_offset = (state->chacha_block_offset + chunk_len) % CHACHA20_BLOCKLENGTH;
        if (state->chacha_block_offset == 0) {
            ChaCha20_increment_counter(&state->ctx);
        }
    }
}
