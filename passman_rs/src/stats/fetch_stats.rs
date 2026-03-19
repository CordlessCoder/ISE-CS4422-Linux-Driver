use std::{
    fs::File,
    io::{self, BufReader, ErrorKind},
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use bstr::{ByteSlice, io::BufReadExt};

#[derive(Debug, Clone)]
pub struct ChaChaSample {
    pub sampled_at: Instant,
    pub data: ChaChaInstant,
}
impl Default for ChaChaSample {
    fn default() -> Self {
        Self {
            sampled_at: Instant::now(),
            data: ChaChaInstant::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChaChaInstant {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub bytes: u64,
    pub reads: u64,
    pub writes: u64,
    pub ioctls: u64,
    pub blocks: u64,
    pub buffered_bytes: u64,
    pub errors: u64,
}

static LAST_SAMPLE: LazyLock<Mutex<ChaChaSample>> =
    LazyLock::new(|| Mutex::new(ChaChaSample::default()));

impl Drop for ChaChaSample {
    fn drop(&mut self) {
        let last = &mut *LAST_SAMPLE.lock().unwrap();
        if last.sampled_at < self.sampled_at {
            core::mem::swap(last, self);
        }
    }
}

pub struct ChaChaDiff {
    pub bytes: u64,
    pub reads: u64,
    pub writes: u64,
    pub ioctls: u64,
    pub blocks: u64,
    pub errors: u64,
    pub over: Duration,
}

impl ChaChaSample {
    pub fn diff_with_last(&self) -> ChaChaDiff {
        let mut last_sample = &*LAST_SAMPLE.lock().unwrap();
        let time_diff = self
            .sampled_at
            .saturating_duration_since(last_sample.sampled_at);
        if last_sample.data == Default::default() {
            last_sample = self;
        }
        ChaChaDiff {
            bytes: self.data.bytes - last_sample.data.bytes,
            reads: self.data.reads - last_sample.data.reads,
            writes: self.data.writes - last_sample.data.writes,
            ioctls: self.data.ioctls - last_sample.data.ioctls,
            blocks: self.data.blocks - last_sample.data.blocks,
            errors: self.data.errors - last_sample.data.errors,
            over: time_diff.max(Duration::from_millis(10)),
        }
    }
    pub fn fetch() -> std::io::Result<Self> {
        let stat = File::open("/proc/chastats")?;
        let mut stat = BufReader::new(stat);
        let mut info = ChaChaSample {
            sampled_at: Instant::now(),
            data: ChaChaInstant::default(),
        };
        stat.for_byte_line(|line| {
            let Some((name, value)) = line.split_once_str(":") else {
                return Ok(false);
            };
            let value = value.trim_ascii();
            let value = core::str::from_utf8(value)
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;
            let value: u64 = value
                .parse()
                .map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;
            match name {
                b"Reads" => info.data.reads = value,
                b"Writes" => info.data.writes = value,
                b"Ioctls" => info.data.ioctls = value,
                b"Blocks" => info.data.blocks = value,
                b"Buffer bytes" => info.data.buffered_bytes = value,
                b"Errors" => info.data.errors = value,
                b"Sessions(Active)" => info.data.active_sessions = value,
                b"Sessions(Total)" => info.data.total_sessions = value,
                b"Bytes Processed" => info.data.bytes = value,
                _ => (),
            }
            Ok(true)
        })?;
        Ok(info)
    }
}
