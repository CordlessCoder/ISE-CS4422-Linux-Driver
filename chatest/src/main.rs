use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::Path,
    sync::atomic::AtomicBool,
    thread,
};

use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr};
use rand::Rng;

const KEY: [u8; 32] = *b"I am a 32-byte key, on god fr fr";
const NONCE: [u8; 8] = *b"12345678";
const BUF_SIZE: usize = 3984;

ioctl_write_ptr!(cha_set_key, 's', 'k', [u8; 32]);
ioctl_write_ptr!(cha_set_nonce, 's', 'n', [u8; 8]);
ioctl_none!(cha_reset_counter, 'r', 'c');
ioctl_write_int!(cha_set_counter, 's', 'c');

static STOP_FLAG: AtomicBool = AtomicBool::new(false);

fn main() {
    unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGTERM, || {
            STOP_FLAG.store(true, std::sync::atomic::Ordering::Relaxed);
        })
        .unwrap();
    };
    let chacha = File::options()
        .write(true)
        .read(true)
        .open("/dev/chacha")
        .unwrap();
    unsafe {
        cha_set_key(chacha.as_raw_fd(), &KEY).unwrap();
        cha_set_nonce(chacha.as_raw_fd(), &NONCE).unwrap();
    }
    let mut chacha = &chacha;
    thread::scope(|s| {
        let mut data = vec![0; BUF_SIZE * 1024];
        let mut output = data.clone();
        rand::rng().fill_bytes(&mut data);
        s.spawn(move || {
            loop {
                chacha.write_all(&data).unwrap();
            }
        });
        while !STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
            _ = chacha.read(&mut output).unwrap();
        }
    });
}
