use std::{
    fs::File,
    io::{Read, Seek},
    os::fd::AsRawFd,
};

use nix::{ioctl_none, ioctl_write_ptr};

ioctl_write_ptr!(cha_set_key, 's', 'k', [u8; 32]);
ioctl_write_ptr!(cha_set_nonce, 's', 'n', [u8; 8]);
ioctl_none!(cha_reset_counter, 'r', 'c');
ioctl_write_ptr!(cha_set_counter, 's', 'c', u64);
ioctl_none!(cha_clear_output_only, 'c', 'o');
ioctl_none!(cha_set_output_only, 's', 'o');

#[derive(Debug)]
pub struct ChaChaDev {
    file: File,
}

impl ChaChaDev {
    #[must_use]
    pub fn open(key: &[u8; 32], nonce: &[u8; 8]) -> Self {
        let file = File::open("/dev/chacha").expect("/dev/chacha unavailable!");
        unsafe {
            cha_set_key(file.as_raw_fd(), key).unwrap();
            cha_set_nonce(file.as_raw_fd(), nonce).unwrap();
            cha_set_output_only(file.as_raw_fd()).unwrap();
        };
        ChaChaDev { file }
    }
    pub fn set_nonce(&mut self, nonce: &[u8; 8]) {
        unsafe {
            cha_set_nonce(self.file.as_raw_fd(), nonce).unwrap();
        }
    }
    pub fn set_key(&mut self, key: &[u8; 32]) {
        unsafe {
            cha_set_key(self.file.as_raw_fd(), key).unwrap();
        }
    }
    pub fn seek(&mut self, offset: u64) {
        self.file.seek(std::io::SeekFrom::Start(offset)).unwrap();
    }
    pub fn reset(&mut self) {
        self.seek(0);
    }
    pub fn apply_keystream(&mut self, mut data: &mut [u8]) {
        let mut buf = [0; 4096];
        while !data.is_empty() {
            let chunk = data.len().min(buf.len());
            self.write_keystream(&mut buf[..chunk]);

            let to_xor = data.split_off_mut(..chunk).unwrap();
            to_xor
                .iter_mut()
                .zip(buf)
                .for_each(|(d, cipher)| *d ^= cipher);
        }
    }
    pub fn write_keystream(&mut self, out: &mut [u8]) {
        self.file.read_exact(out).unwrap();
    }
}
