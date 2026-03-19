use std::{fs, thread, time::Duration};

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub reads: u64,
    pub writes: u64,
    pub blocks: u64,
    pub errors: u64,
    pub ioctls: u64,
    pub current_buffer_bytes: u64,
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub bytes_processed: u64,
}

pub fn read_stats() -> Stats {
    let content = std::fs::read_to_string("/proc/chacha_stats")
        .expect("Failed to read /proc/chacha_stats");

    let mut stats = Stats {
        reads: 0,
        writes: 0,
        blocks: 0,
        errors: 0,
        ioctls: 0,
        current_buffer_bytes: 0,
        total_sessions: 0,
        active_sessions: 0,
        bytes_processed: 0,
    };

    for line in content.lines() {
        if let Some(v) = line.strip_prefix("reads:") {
            stats.reads = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("writes:") {
            stats.writes = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("blocks:") {
            stats.blocks = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("errors:") {
            stats.errors = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("ioctls:") {
            stats.ioctls = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("current_buffer_bytes:") {
            stats.current_buffer_bytes = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("total_sessions:") {
            stats.total_sessions = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("active_sessions:") {
            stats.active_sessions = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("bytes_processed:") {
            stats.bytes_processed = v.trim().parse().unwrap();
        }
    }

    stats
}

pub fn poll_stats(interval: Duration) {
    let mut prev = read_stats();

    loop {
        thread::sleep(interval);

        let current = read_stats();

        // compute rates
        let reads_per_sec = current.reads - prev.reads;
        let writes_per_sec = current.writes - prev.writes;
        let ioctls_per_sec = current.ioctls - prev.ioctls;
        let bytes_per_sec = current.bytes_processed - prev.bytes_processed;
        let errors_per_sec = current.errors - prev.errors;

        println!(
            "R/s: {} | W/s: {} | IOCTL/s: {}",
            reads_per_sec, writes_per_sec, ioctls_per_sec
        );

        prev = current;
    }
}
