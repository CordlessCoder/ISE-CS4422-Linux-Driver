#ifndef CHACHA20_H
#define CHACHA20_H

#include "linux-crypt.h"
#include <linux/string.h>

#define CHACHA20_KEYSIZE 256
#define CHACHA20_BLOCKLENGTH 64

typedef struct {
    u32 input[16];
} ChaCha20Ctx;

void ChaCha20_set_key(ChaCha20Ctx* ctx, const u8 k[32]);
void ChaCha20_set_nonce(ChaCha20Ctx* ctx, const u8 nonce[8]);
void ChaCha20_set_counter(ChaCha20Ctx* ctx, u64 counter);
void ChaCha20_increment_counter(ChaCha20Ctx* ctx);
u64 ChaCha20_get_counter(ChaCha20Ctx* crx);
void ChaCha20_xor(ChaCha20Ctx* ctx, u8* data, size_t bytes);
void ChaCha20_xorblock_noinc(ChaCha20Ctx* ctx, u8 data[CHACHA20_BLOCKLENGTH]);
#endif
