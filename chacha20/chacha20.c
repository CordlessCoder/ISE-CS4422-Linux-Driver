#include "linux-crypt.h"
#include <linux/string.h>
/*
chacha-ref.c version 20080118
D. J. Bernstein
Public domain.
*/

#define CHACHA20_KEYSIZE 256

typedef struct {
    u32 input[16];
} ChaCha20Ctx;

/*
 * Encryption/decryption of arbitrary length messages.
 *
 * For efficiency reasons, the API provides two types of
 * encrypt/decrypt functions. The ECRYPT_encrypt_bytes() function
 * (declared here) encrypts byte strings of arbitrary length, while
 * the ECRYPT_encrypt_blocks() function (defined later) only accepts
 * lengths which are multiples of ECRYPT_BLOCKLENGTH.
 *
 * The user is allowed to make multiple calls to
 * ECRYPT_encrypt_blocks() to incrementally encrypt a long message,
 * but he is NOT allowed to make additional encryption calls once he
 * has called ECRYPT_encrypt_bytes() (unless he starts a new message
 * of course). For example, this sequence of calls is acceptable:
 *
 * ECRYPT_keysetup();
 *
 * ECRYPT_ivsetup();
 * ECRYPT_encrypt_blocks();
 * ECRYPT_encrypt_blocks();
 * ECRYPT_encrypt_bytes();
 *
 * ECRYPT_ivsetup();
 * ECRYPT_encrypt_blocks();
 * ECRYPT_encrypt_blocks();
 *
 * ECRYPT_ivsetup();
 * ECRYPT_encrypt_bytes();
 *
 * The following sequence is not:
 *
 * ECRYPT_keysetup();
 * ECRYPT_ivsetup();
 * ECRYPT_encrypt_blocks();
 * ECRYPT_encrypt_bytes();
 * ECRYPT_encrypt_blocks();
 */

#define CHACHA20_BLOCKLENGTH 64

#define ROTATE(v, c) (ROTL32(v, c))
#define XOR(v, w) ((v) ^ (w))
#define PLUS(v, w) (U32V((v) + (w)))
#define PLUSONE(v) (PLUS((v), 1))

#define QUARTERROUND(a, b, c, d)                                                                                                                                                                       \
    x[a] = PLUS(x[a], x[b]);                                                                                                                                                                           \
    x[d] = ROTATE(XOR(x[d], x[a]), 16);                                                                                                                                                                \
    x[c] = PLUS(x[c], x[d]);                                                                                                                                                                           \
    x[b] = ROTATE(XOR(x[b], x[c]), 12);                                                                                                                                                                \
    x[a] = PLUS(x[a], x[b]);                                                                                                                                                                           \
    x[d] = ROTATE(XOR(x[d], x[a]), 8);                                                                                                                                                                 \
    x[c] = PLUS(x[c], x[d]);                                                                                                                                                                           \
    x[b] = ROTATE(XOR(x[b], x[c]), 7);


static void salsa20_wordtobyte(u8 output[CHACHA20_BLOCKLENGTH], const u32 input[16]);
void ChaCha20_set_key(ChaCha20Ctx* ctx, const u8 k[32]);
void ChaCha20_set_nonce(ChaCha20Ctx* ctx, const u8 nonce[8]);
void ChaCha20_set_counter(ChaCha20Ctx* ctx, u64 counter);
void ChaCha20_increment_counter(ChaCha20Ctx* ctx);
u64 ChaCha20_get_counter(ChaCha20Ctx* crx);
void ChaCha20_xor(ChaCha20Ctx* ctx, u8* data, size_t bytes);
void ChaCha20_xorblock_noinc(ChaCha20Ctx* ctx, u8 data[CHACHA20_BLOCKLENGTH]);

static void salsa20_wordtobyte(u8 output[CHACHA20_BLOCKLENGTH], const u32 input[16]) {
    u32 x[16] = {};
    memcpy(x, input, sizeof(x));

    for (int i = 20; i > 0; i -= 2) {
        QUARTERROUND(0, 4, 8, 12)
        QUARTERROUND(1, 5, 9, 13)
        QUARTERROUND(2, 6, 10, 14)
        QUARTERROUND(3, 7, 11, 15)
        QUARTERROUND(0, 5, 10, 15)
        QUARTERROUND(1, 6, 11, 12)
        QUARTERROUND(2, 7, 8, 13)
        QUARTERROUND(3, 4, 9, 14)
    }
    for (int i = 0; i < 16; ++i)
        x[i] = PLUS(x[i], input[i]);
    for (int i = 0; i < 16; ++i)
        cpu_to_le64s(&x[i]);
    memcpy(output, x, sizeof(x));
}

void ChaCha20_set_key(ChaCha20Ctx* ctx, const u8 k[32]) {
    u32 key[8] = {};
    memcpy(key, k, sizeof(key));
    ctx->input[4] = le32_to_cpup(key + 0);
    ctx->input[5] = le32_to_cpup(key + 1);
    ctx->input[6] = le32_to_cpup(key + 2);
    ctx->input[7] = le32_to_cpup(key + 3);
    ctx->input[8] = le32_to_cpup(key + 4);
    ctx->input[9] = le32_to_cpup(key + 5);
    ctx->input[10] = le32_to_cpup(key + 6);
    ctx->input[11] = le32_to_cpup(key + 7);
    const u32 sigma[4] = {0x61707865, 0x3320646e, 0x79622d32, 0x6b206574};
    ctx->input[0] = sigma[0];
    ctx->input[1] = sigma[1];
    ctx->input[2] = sigma[2];
    ctx->input[3] = sigma[3];
    ChaCha20_set_counter(ctx, 0);
}

void ChaCha20_set_nonce(ChaCha20Ctx* ctx, const u8 nonce[8]) {
    u32 n[2] = {};
    memcpy(n, nonce, sizeof(n));
    ctx->input[12] = 0;
    ctx->input[13] = 0;
    ctx->input[14] = le32_to_cpup(n + 0);
    ctx->input[15] = le32_to_cpup(n + 1);
    ChaCha20_set_counter(ctx, 0);
}

void ChaCha20_set_counter(ChaCha20Ctx* ctx, u64 counter) {
    cpu_to_le64s(&counter);
    ctx->input[12] = counter >> 32;
    ctx->input[13] = U32V(counter);
}

u64 ChaCha20_get_counter(ChaCha20Ctx* ctx) { return ((u64)ctx->input[13] << 32) | (((u64)ctx->input[12])); }

void ChaCha20_increment_counter(ChaCha20Ctx* ctx) {
    ctx->input[12] = PLUSONE(ctx->input[12]);
    if (!ctx->input[12]) {
        ctx->input[13] = PLUSONE(ctx->input[13]);
    }
}

void ChaCha20_xorblock_noinc(ChaCha20Ctx* ctx, u8 data[CHACHA20_BLOCKLENGTH]) {
    u8 output[CHACHA20_BLOCKLENGTH];
    int i;

    salsa20_wordtobyte(output, ctx->input);
    for (i = 0; i < CHACHA20_BLOCKLENGTH; ++i)
        data[i] ^= output[i];
}

void ChaCha20_xor(ChaCha20Ctx* ctx, u8* data, size_t bytes) {
    u8 output[CHACHA20_BLOCKLENGTH] = {};

    while (bytes) {
        size_t chunk = (bytes < CHACHA20_BLOCKLENGTH) ? bytes : CHACHA20_BLOCKLENGTH;
        memcpy(output, data, chunk);
        ChaCha20_xorblock_noinc(ctx, output);
        ChaCha20_increment_counter(ctx);
        memcpy(data, output, chunk);
        bytes -= CHACHA20_BLOCKLENGTH;
        data += CHACHA20_BLOCKLENGTH;
    }
}
