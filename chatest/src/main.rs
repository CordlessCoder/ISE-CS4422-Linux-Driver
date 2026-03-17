use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    path::Path,
};

fn main() {
    let mut chacha = File::options()
        .write(true)
        .read(true)
        .open("/dev/chacha")
        .unwrap();
    let mut buf = *b"foobar";
    println!("Original: {}", Path::new(OsStr::from_bytes(&buf)).display());
    chacha.write_all(&buf).unwrap();
    chacha.read_exact(&mut buf).unwrap();
    println!("Encrypted: {buf:?}",);
    let mut chacha = File::options()
        .write(true)
        .read(true)
        .open("/dev/chacha")
        .unwrap();
    chacha.write_all(&buf).unwrap();
    chacha.read_exact(&mut buf).unwrap();
    println!(
        "Decrypted: {}",
        Path::new(OsStr::from_bytes(&buf)).display()
    );
}
