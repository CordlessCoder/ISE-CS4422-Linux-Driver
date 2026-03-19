use clap::Parser;
use nix::{ioctl_none, ioctl_write_ptr};
use std::{
    fs::File,
    io::{Read, Write},
    os::fd::AsRawFd,
    sync::atomic::AtomicBool,
};

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

/// A benchmark to show the peak throughput of /dev/chacha
#[derive(Parser)]
struct Cli {
    /// Use the read-only optimization to read the stream cipher straight from the device
    #[arg(short, long, default_value_t)]
    read_only_opt: bool,
    /// Number of threads to spawn
    #[arg(short, long, default_value_t = 1)]
    parallelism: u32,
}

fn main() {
    unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGTERM, || {
            STOP_FLAG.store(true, std::sync::atomic::Ordering::Relaxed);
        })
        .unwrap();
    };
    let cli = Cli::parse();

    std::thread::scope(|s| {
        for _ in 0..cli.parallelism {
            s.spawn(|| {
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
                if cli.read_only_opt {
                    unsafe {
                        cha_set_output_only(chacha.as_raw_fd()).unwrap();
                    }
                    let mut buf = vec![0; BUF_SIZE * 1024];
                    while !STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
                        _ = chacha.read(&mut buf).unwrap();
                    }
                } else {
                    let mut buf = [0; BUF_SIZE];
                    while !STOP_FLAG.load(std::sync::atomic::Ordering::Relaxed) {
                        chacha.write_all(&buf).unwrap();
                        chacha.read_exact(&mut buf).unwrap();
                    }
                }
            });
        }
    })
}
