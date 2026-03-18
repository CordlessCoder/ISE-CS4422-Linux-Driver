use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::Path,
};

use chacha20::{
    KeyIvInit,
    cipher::{Array, StreamCipher, StreamCipherSeek},
};
use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr};

const DATA: [u8; 6] = *b"foobar";
const KEY: [u8; 32] = *b"I am a 32-byte key, on god fr fr";
const NONCE: [u8; 8] = *b"12345678";

// #define SET_KEY _IOW('s', 'k', char[32])
// #define SET_NONCE _IOW('s', 'n', char[8])
// #define RESET_COUNTER _IO('r', 'c')
// #define SET_COUNTER _IOW('s', 'c', int*)
ioctl_write_ptr!(cha_set_key, 's', 'k', [u8; 32]);
ioctl_write_ptr!(cha_set_nonce, 's', 'n', [u8; 8]);
ioctl_none!(cha_reset_counter, 'r', 'c');
ioctl_write_int!(cha_set_counter, 's', 'c');

fn main() {
    let mut chacha = File::options()
        .write(true)
        .read(true)
        .open("/dev/chacha")
        .unwrap();
    unsafe {
        let mut key = KEY;
        let mut nonce = NONCE;
        cha_set_key(chacha.as_raw_fd(), &key).unwrap();
        cha_set_nonce(chacha.as_raw_fd(), &nonce).unwrap();
    }
    let mut reference = chacha20::ChaCha20Legacy::new(&Array(KEY), &Array(NONCE));

    let mut ref_buf = DATA;
    let mut buf = DATA;
    println!("Original: {}", Path::new(OsStr::from_bytes(&buf)).display());

    chacha.write_all(&buf).unwrap();
    chacha.read_exact(&mut buf).unwrap();

    reference.apply_keystream(&mut ref_buf);

    println!("Encrypted: {buf:?}",);
    println!("Encrypted(ref): {ref_buf:?}",);

    unsafe {
        cha_reset_counter(chacha.as_raw_fd()).unwrap();
    }

    reference.seek(0);
    chacha.write_all(&buf).unwrap();
    chacha.read_exact(&mut buf).unwrap();
    reference.apply_keystream(&mut ref_buf);
    println!(
        "Decrypted: {}",
        Path::new(OsStr::from_bytes(&buf)).display()
    );
    println!(
        "Decrypted(ref): {}",
        Path::new(OsStr::from_bytes(&ref_buf)).display()
    );
}
