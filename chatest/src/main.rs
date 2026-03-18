use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::Path,
};

use chacha20::{
    KeyIvInit,
    cipher::{Array, StreamCipher, StreamCipherSeek},
};
use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr};

const DATA: &str = "This is just an example message, repeated a couple of times
This is just an example message, repeated a couple of times
This is just an example message, repeated a couple of times
";
const KEY: [u8; 32] = *b"I am a 32-byte key, on god fr fr";
const NONCE: [u8; 8] = *b"12345678";

ioctl_write_ptr!(cha_set_key, 's', 'k', [u8; 32]);
ioctl_write_ptr!(cha_set_nonce, 's', 'n', [u8; 8]);
ioctl_none!(cha_reset_counter, 'r', 'c');
ioctl_write_int!(cha_set_counter, 's', 'c');

fn test_message(message: &str) {
    println!("Testing message {message}");
    let mut chacha = File::options()
        .write(true)
        .read(true)
        .open("/dev/chacha")
        .unwrap();
    unsafe {
        cha_set_key(chacha.as_raw_fd(), &KEY).unwrap();
        cha_set_nonce(chacha.as_raw_fd(), &NONCE).unwrap();
    }
    let mut reference = chacha20::ChaCha20Legacy::new(&Array(KEY), &Array(NONCE));

    let mut ref_buf = message.as_bytes().to_vec();
    let mut buf = ref_buf.clone();
    println!("Original: {}", Path::new(OsStr::from_bytes(&buf)).display());

    let third = buf.len() / 3;
    std::thread::scope(|s| {
        let mut chacha = &chacha;
        let read_from = buf.clone();
        s.spawn(move || {
            chacha.write_all(&read_from).unwrap();
        });
        chacha.read_exact(&mut buf[..third]).unwrap();
        chacha.read_exact(&mut buf[third..]).unwrap();
    });

    reference.apply_keystream(&mut ref_buf);

    println!("Encrypted: {buf:?}",);
    println!("Encrypted(ref): {ref_buf:?}",);
    if buf != ref_buf {
        std::fs::write("eref", &ref_buf).unwrap();
        std::fs::write("egot", &buf).unwrap();
    }
    assert_eq!(buf, ref_buf);

    chacha.seek(SeekFrom::Start(0)).unwrap();
    reference.seek(0);
    std::thread::scope(|s| {
        let mut chacha = &chacha;
        let read_from = buf.clone();
        s.spawn(move || {
            chacha.write_all(&read_from).unwrap();
        });
        chacha.read_exact(&mut buf[..third]).unwrap();
        chacha.read_exact(&mut buf[third..]).unwrap();
    });
    reference.apply_keystream(&mut ref_buf);
    println!(
        "Decrypted: {}",
        Path::new(OsStr::from_bytes(&buf)).display()
    );
    println!(
        "Decrypted(ref): {}",
        Path::new(OsStr::from_bytes(&ref_buf)).display()
    );
    if buf != ref_buf {
        std::fs::write("ref", &ref_buf).unwrap();
        std::fs::write("got", &buf).unwrap();
    }
    assert_eq!(buf, ref_buf);
}

fn main() {
    test_message(&DATA[..20]);
    test_message(&DATA[..119]);
    let repeated = DATA.repeat(23);
    for rep in 1..10000 {
        test_message(&repeated);
        test_message(&DATA.repeat(rep));
    }
}
