Vault file structure:
-unencrypted- size: 32 bytes long
"CVLT"      | fixed 4-byte magic number
NONCE       | 8-byte nonce(little-endian), randomized on each encryption
BLAKE2B_HASH| 20-byte hash of the following fields, in order:
            | [CHACHA KEY, NONCE, DATA(unencrypted), LEN(unencrypted)]
--encrypted-- size: `(4096 - 32) + 4096k` bytes long, where `k` is some non-negative integer
LEN         | The length of the actual data stored(64-bit unsigned integer, little-endian)
DATA        | The data stored in the vault, encrypted with ChaCha20
PADDING     | Zeroes padding the size of the vault to a multiple of 4096 bytes

The header is useful as an extra check to ensure the vault file is in fact a vault.

The nonce is used as the nonce for ChaCha20.
Randomizing the nonce every encryption is necessary to avoid key+nonce reuse, as that results in
the secret being XORed with identical ciphers - allowing the attacker to "cancel out" the cipher
in two inputs.

The hash of the unencrypted data provides authentication - the only way for an entity to produce
a hash that matches the unencrypted state of the vault, is to know the unencrypted state of the
vault - which is only true if they know the key.

The extreme padding is used to avoid leaking the precise length of the encrypted data to
external observers.

# An overview of the encryption/decryption pipeline
When unlocking:
- Load the nonce, to use as a salt for argon2
- Hash the password, providing the key and nonce to ChaCha20
- Decrypt and then hash(blake2b) the entire file
- Ensure the BLAKE2_HASH field matches the computed hash
  (if it doesn't, the encrypted data was tampered with or the password didn't match)
Provide the decrypted data to the user

When locking:
- (OPTIONAL) Unlock the existing vault to ensure there hasn't been a tamper attempt
  take note: if the existing state of the vault has been authentified, the nonce can be
  incremented instead of randomly generating a new one
- Generate random nonce(or increment existing nonce, see above note) and store it
- Hash the password, providing the key and nonce to ChaCha20
- Hash the data(blake2b), and store the hash in BLAKE2_HASH
- Encrypt the data, and store it in DATA
