use crate::chacha_dev::ChaChaDev;
use blake2::Digest;
use rand::TryRng;
use std::{
    fs::File,
    io::{self, Read, Seek, Write},
    path::Path,
};
use zeroize::Zeroizing;

pub mod chacha_dev;
pub mod cli;

const PEPPER: &[u8] = b"CVLT_FIXED_SALT";

const KEY_SIZE: usize = 32;
const MAGIC: [u8; 4] = *b"CVLT";
const NONCE_SIZE: usize = 8;
const HEADER_SIZE: usize = 32;
const BLAKE2_HASH_SIZE: usize = 20;
const LEN_SIZE: usize = 8;
const NONCE_OFFSET: usize = 4;
const HASH_OFFSET: usize = 12;
const LEN_OFFSET: usize = 32;
const DATA_OFFSET: usize = 40;
const PAD_TO: usize = 4096;

type Blake2b = blake2::Blake2b<blake2::digest::typenum::U20>;

fn get_argon2() -> argon2::Argon2<'static> {
    argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::ParamsBuilder::new()
            .m_cost(47104)
            .t_cost(1)
            .p_cost(1)
            .output_len(KEY_SIZE)
            .build()
            .unwrap(),
    )
}

#[derive(Debug)]
pub struct ClosedVault {
    file: File,
    nonce: u64,
    hash: [u8; 20],
}

#[derive(Debug)]
pub struct OpenVault {
    closed: ClosedVault,
    key: Zeroizing<[u8; KEY_SIZE]>,
    len: u64,
    chacha: ChaChaDev,
}

pub struct OpenVaultReader<'v> {
    vault: &'v mut OpenVault,
    blake: Blake2b,
    offset: u64,
    hashed_up_to: u64,
}

pub struct OpenVaultWriter<'v> {
    vault: &'v mut OpenVault,
    blake: Blake2b,
    finalized_at_len: Option<u64>,
    chacha_cache: [u8; 4096],
}

impl Drop for OpenVaultWriter<'_> {
    fn drop(&mut self) {
        _ = self.update_header();
    }
}

impl OpenVaultWriter<'_> {
    fn seek_to_offset(&mut self) -> io::Result<()> {
        self.vault
            .closed
            .file
            .seek(io::SeekFrom::Start(DATA_OFFSET as u64 + self.vault.len))?;
        Ok(())
    }
    pub fn update_header(&mut self) -> io::Result<()> {
        if self.finalized_at_len == Some(self.vault.len) {
            return Ok(());
        }
        // TODO: Make sure to write the hash, nonce etc.
        let mut hash: [u8; BLAKE2_HASH_SIZE] = [0u8; _];
        let mut blake_final = self.blake.clone();
        blake_final.update(self.vault.len.to_le_bytes());
        blake_final.finalize_into(blake2::digest::generic_array::GenericArray::from_mut_slice(
            &mut hash,
        ));

        let mut header = [0u8; HEADER_SIZE + LEN_SIZE];
        header[..MAGIC.len()].copy_from_slice(&MAGIC);
        header[NONCE_OFFSET..][..NONCE_SIZE]
            .copy_from_slice(&self.vault.closed.nonce.to_le_bytes());
        header[HASH_OFFSET..][..BLAKE2_HASH_SIZE].copy_from_slice(&hash);
        {
            // Write and encrypt the length
            let len_bytes = &mut header[LEN_OFFSET..][..LEN_SIZE];
            len_bytes.copy_from_slice(&self.vault.len.to_le_bytes());
            self.vault.chacha.seek(0);
            self.vault.chacha.apply_keystream(len_bytes);
            // Reset chacha back to being 4096 bytes ahead of offset
            self.vault
                .chacha
                .seek(DATA_OFFSET as u64 + self.vault.len + self.chacha_cache.len() as u64);
        }
        self.vault.closed.file.seek(io::SeekFrom::Start(0))?;
        self.vault.closed.file.write_all(&header)?;
        // Write out the zero padding
        self.seek_to_offset()?;
        let cur_size = self.vault.len + DATA_OFFSET as u64;
        let padded_size = cur_size.next_multiple_of(PAD_TO as u64);
        let padding = padded_size - cur_size;
        self.vault
            .closed
            .file
            .write_all(&self.chacha_cache[..padding as usize])?;
        self.seek_to_offset()?;
        self.finalized_at_len = Some(self.vault.len);
        Ok(())
    }
}

impl io::Write for OpenVaultWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // TODO: Hash, encrypt and write out the data
        let mut scratch = self.chacha_cache;
        let chunk = buf.len().min(scratch.len());
        scratch
            .iter_mut()
            .zip(buf)
            .for_each(|(cipher, data)| *cipher ^= data);
        let wrote = self.vault.closed.file.write(&scratch[..chunk])?;
        // Update cache
        let remaining_cache = self.chacha_cache.len() - wrote;
        self.chacha_cache.rotate_left(wrote);
        self.vault
            .chacha
            .write_keystream(&mut self.chacha_cache[remaining_cache..]);

        self.blake.update(&buf[..wrote]);
        self.vault.len += wrote as u64;

        Ok(wrote)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.vault.closed.file.flush()
    }
}

impl OpenVaultReader<'_> {
    pub fn authenticate(&mut self) -> bool {
        io::copy(self, &mut io::sink()).unwrap();
        let mut hash: [u8; BLAKE2_HASH_SIZE] = [0u8; _];
        let mut blake_final = self.blake.clone();
        blake_final.update(self.vault.len.to_le_bytes());
        blake_final.finalize_into(blake2::digest::generic_array::GenericArray::from_mut_slice(
            &mut hash,
        ));
        hash == self.vault.closed.hash
    }
}

impl io::Read for OpenVaultReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let got = (&mut self.vault.closed.file)
            .take(self.vault.len.saturating_sub(self.offset))
            .read(buf)?;
        self.vault.chacha.apply_keystream(&mut buf[..got]);
        let unhashed_start = self.hashed_up_to.saturating_sub(self.offset).try_into().expect("the read should not overflow a pointer");
        self.blake
            .update(buf.get(unhashed_start..got).unwrap_or_default());
        self.offset += got as u64;
        self.hashed_up_to = self.hashed_up_to.max(self.offset);
        Ok(got)
    }
}

impl io::Seek for OpenVaultReader<'_> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        use io::SeekFrom::{Start, End, Current};
        match pos {
            Start(start) => {
                let start = start.min(self.vault.len);
                self.offset = start;
            }
            End(end) => {
                self.offset = self.vault.len.saturating_sub(end.cast_unsigned());
            }
            Current(offset) => {
                self.offset =
                    ((self.offset.cast_signed() + offset).cast_unsigned()).min(self.vault.len);
            }
        }
        // NOTE: ChaCha20 seeks must be offset by the size of length
        self.vault.chacha.seek(LEN_SIZE as u64 + self.offset);
        self.vault
            .closed
            .file
            .seek(Start(DATA_OFFSET as u64 + self.offset))
    }
}

impl ClosedVault {
    pub fn open(path: &Path) -> io::Result<Self> {
        let mut file = File::options().write(true).read(true).open(path)?;
        let mut header: [u8; HEADER_SIZE] = [0; _];
        file.read_exact(&mut header)?;
        if header[..MAGIC.len()] != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "vault file missing vault magic number",
            ));
        }
        let nonce = u64::from_le_bytes(header[NONCE_OFFSET..][..NONCE_SIZE].try_into().unwrap());
        let hash: [u8; BLAKE2_HASH_SIZE] = header[HASH_OFFSET..][..BLAKE2_HASH_SIZE]
            .try_into()
            .unwrap();
        Ok(ClosedVault { file, nonce, hash })
    }
    pub fn unlock(mut self, password: &[u8]) -> io::Result<OpenVault> {
        let kdf = get_argon2();

        let mut key = Zeroizing::new([0u8; KEY_SIZE]);
        kdf.hash_password_into(password, PEPPER, &mut *key).unwrap();

        let mut chacha = ChaChaDev::open(&key, &self.nonce.to_le_bytes());
        let mut len_bytes = [0u8; 8];
        self.file.seek(io::SeekFrom::Start(LEN_OFFSET as u64))?;
        self.file.read_exact(&mut len_bytes)?;

        // Decrypt the length
        chacha.apply_keystream(&mut len_bytes);
        let len = u64::from_le_bytes(len_bytes);

        Ok(OpenVault {
            closed: self,
            key,
            len,
            chacha,
        })
    }
}

impl OpenVault {
    pub fn create(path: &Path, password: &[u8]) -> io::Result<Self> {
        let mut file = File::options()
            .create_new(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(path)?;
        let mut empty_vault: [u8; 4096] = [0; 4096];
        empty_vault[..NONCE_OFFSET].copy_from_slice(&MAGIC);
        let nonce: u64 = rand::rngs::SysRng.try_next_u64()?;
        let kdf = get_argon2();

        let nonce_bytes = nonce.to_le_bytes();
        empty_vault[NONCE_OFFSET..][..NONCE_SIZE].copy_from_slice(&nonce_bytes);

        let mut key = Zeroizing::new([0u8; KEY_SIZE]);
        kdf.hash_password_into(password, PEPPER, &mut *key).unwrap();

        let mut hash: [u8; BLAKE2_HASH_SIZE] = [0u8; _];
        let mut hasher = Blake2b::new();
        // Get hash of empty vault, so there's no data to hash
        hasher.update(&key);
        hasher.update(nonce_bytes);
        hasher.update(0u64.to_le_bytes());
        hasher.finalize_into(blake2::digest::generic_array::GenericArray::from_mut_slice(
            &mut hash,
        ));

        empty_vault[HASH_OFFSET..][..BLAKE2_HASH_SIZE].copy_from_slice(&hash);

        let mut chacha = ChaChaDev::open(&key, &nonce_bytes);
        // We can write the keystream directly without XORing,
        // since we'd be XORing with zeroes anyway
        chacha.write_keystream(&mut empty_vault[HEADER_SIZE..]);

        file.write_all(&empty_vault)?;
        Ok(OpenVault {
            closed: ClosedVault { file, nonce, hash },
            key,
            chacha,
            len: 0,
        })
    }
    pub fn authenticate(&mut self) -> bool {
        self.get_reader().authenticate()
    }
    pub fn get_reader(&mut self) -> OpenVaultReader<'_> {
        self.closed
            .file
            .seek(io::SeekFrom::Start(DATA_OFFSET as u64))
            .unwrap();
        self.chacha.seek(LEN_SIZE as u64);
        let mut blake = Blake2b::new();
        blake.update(&self.key);
        blake.update(self.closed.nonce.to_le_bytes());
        OpenVaultReader {
            vault: self,
            blake,
            offset: 0,
            hashed_up_to: 0,
        }
    }
    pub fn truncate_and_get_writer(&mut self) -> OpenVaultWriter<'_> {
        // Destroy the encrypted data as it's about to be overwritten
        self.closed.file.set_len(DATA_OFFSET as u64).unwrap();
        self.closed
            .file
            .seek(io::SeekFrom::Start(DATA_OFFSET as u64))
            .unwrap();
        self.len = 0;
        // Increment nonce
        self.closed.nonce = self.closed.nonce.wrapping_add(1);

        // Update chacha
        self.chacha.set_nonce(&self.closed.nonce.to_le_bytes());
        self.chacha.seek(LEN_SIZE as u64);

        let mut blake = Blake2b::new();
        blake.update(&self.key);
        blake.update(self.closed.nonce.to_le_bytes());
        let mut chacha_cache = [0; _];
        self.chacha.write_keystream(&mut chacha_cache);
        OpenVaultWriter {
            vault: self,
            blake,
            finalized_at_len: None,
            chacha_cache,
        }
    }
}
