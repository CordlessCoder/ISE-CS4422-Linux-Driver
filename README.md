# ISE-CS4422-Linux-Driver

A simple password manager with encryption implemented in kernel-space as a device driver.

The encryption will be implemented using ChaCha20,
following the [OWASP Secrets Management guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html#71-encryption-types-to-use) .

Key derivation will be implemented using [Argon2id with the parameters `m=47104, t=1, p=1`](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id).
NOTE: Make sure to set argon2id to generate a 256-bit(32-byte) key, since that's what ChaCha20 expects.

## Driver Usage Example
```c
passphrase = "I AM AN EXAMPLE PASSPHRASE"
data_to_be_encrypted = "some_bytes"
// The nonce, needs to be stored with the encrypted data to recover the original.
nonce = cryptographic_rng.generate(8 bytes)

// fixed hashing setup
kdf = argon2id.from_parameters(m=47104, t=1, p=1)

// Key derivation(turns the password into the 256-bit key)
key = kdf.hash(passphrase)

encryption_device = open("/dev/chacha")
ioctl(encryption_device, SET_KEY, &key)
ioctl(encryption_device, SET_NONCE, &nonce)

// Hand the data over to the device to be encrypted
write(encryption_device, data_to_be_encrypted)

encrypted_result = read(encryption_device)

// Undo the encryption by passing the data through the device again
ioctl(encryption_device, RESET_COUNTER)

write(encryption_device, encrypted_result)
decrypted_data = read(encryption_device)

assert(decrypted_data == data_to_be_encrypted)
close(encryption_device)
```

## Tasks
- [x] ChaCha20 device driver(`/dev/chacha`)
- [ ] Command-Line Interface password manager
- [ ] Graphical Interface
- [x] Statistics exposed via procfs(`/proc/chacha`)
- [ ] Live procfs statistic visualization
- [ ] DevOps: Compiling everything in a github action
- [ ] DevOps: Linting with `clang-tidy`
