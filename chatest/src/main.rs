use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    path::Path,
};

use chacha20::{
    KeyIvInit,
    cipher::{Array, StreamCipher},
};

const DATA: [u8; 6] = *b"foobar";

fn main() {
    let mut chacha = File::options()
        .write(true)
        .read(true)
        .open("/dev/chacha")
        .unwrap();
    let mut reference = chacha20::ChaCha20::new(&Array([0; _]), &Array([0; _]));

    let mut ref_buf = DATA;
    let mut buf = DATA;
    println!("Original: {}", Path::new(OsStr::from_bytes(&buf)).display());

    chacha.write_all(&buf).unwrap();
    chacha.read_exact(&mut buf).unwrap();
    reference.apply_keystream(&mut ref_buf);

    println!("Encrypted: {buf:?}",);
    println!("Encrypted(ref): {ref_buf:?}",);

    let mut chacha = File::options()
        .write(true)
        .read(true)
        .open("/dev/chacha")
        .unwrap();
    chacha.write_all(&buf).unwrap();
    chacha.read_exact(&mut buf).unwrap();
    reference.apply_keystream(&mut ref_buf);
    println!(
        "Decrypted: {}",
        Path::new(OsStr::from_bytes(&buf)).display()
    );
    println!(
        "Decrypted(ref): {}",
        Path::new(OsStr::from_bytes(&buf)).display()
    );
}
