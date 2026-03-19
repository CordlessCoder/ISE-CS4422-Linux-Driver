use std::{
    fs::File,
    io::Read,
    os::fd::AsRawFd,
    sync::atomic::AtomicBool,
};
use nix::{ioctl_none, ioctl_write_ptr};

const KEY: [u8; 32] = *b"I am a 32-byte key, on god fr fr";
const NONCE: [u8; 8] = *b"12345678";
const BUF_SIZE: usize = 3984;

ioctl_write_ptr!(cha_set_key, 's', 'k', [u8; 32]);
ioctl_write_ptr!(cha_set_nonce, 's', 'n', [u8; 8]);
ioctl_none!(cha_reset_counter, 'r', 'c');
ioctl_write_ptr!(cha_set_counter, 's', 'c', u64);
ioctl_none!(cha_clear_output_only, 'c', 'o');
ioctl_none!(cha_set_output_only, 's', 'o');

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
        cha_set_output_only(chacha.as_raw_fd()).unwrap();
    }
    let mut chacha = &chacha;
    let mut buf = vec![0; BUF_SIZE * 1024];
    while !STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
        _ = chacha.read(&mut buf).unwrap();
    }
}
