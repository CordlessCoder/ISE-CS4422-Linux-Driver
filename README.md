# ISE-CS4422-Linux-Driver

A simple password manager with encryption implemented in kernel-space as a device driver.

The encryption will be implemented using AES-256 in GCM mode,
following the [OWASP Cryptographic Storage guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html).

Key derivation will be implemented using [Argon2id with the parameters `m=47104, t=1, p=1`](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id).
NOTE: Make sure to set argon2id to generate a 256-bit(32-byte) key, since that's what AES-256 expects.

## Driver Usage Example
```c
passphrase = "I AM AN EXAMPLE PASSPHRASE"
data_to_be_encrypted = "some_bytes"

// fixed hashing setup
kdf = argon2id.from_parameters(m=47104, t=1, p=1)
salt = "this must be some fixed byte-string"

// Key derivation(turns the password into the 256-bit key)
key = kdf.hash(passphrase)

aes_device = open("/dev/aes")
ioctl(aes_device, SET_KEY, &key)
action = AES_ENCRYPT
ioctl(aes_device, SET_AES_ACTION, &action)

// Hand the data over to the device to be encrypted
write(aes_device, data_to_be_encrypted)

encrypted_result = read(aes_device)

// Undo the encryption using the key
action = AES_DECRYPT
ioctl(aes_device, SET_AES_ACTION, &action)

write(aes_device, encrypted_result)
decrypted_data = read(aes_device)

assert(decrypted_data == data_to_be_encrypted)
close(aes_device)
```

## Tasks
- [ ] AES-256-GCM device driver(`/dev/aes`)
- [ ] Command-Line Interface password manager
- [ ] Graphical Interface
- [ ] Statistics exposed via procfs(`/proc/aes`)
- [ ] Live procfs statistic visualization
- [ ] DevOps: Compiling everything in a github action
- [ ] DevOps: Linting with `clang-tidy`
