use std::{fs, thread, time::Duration};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Chart, Dataset, Axis},
    layout::{Layout, Constraint, Direction},
};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode},
    execute,
};
use std::{io, time::{Duration, Instant}};

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
        if let Some(v) = line.strip_prefix("Reads=") {
            stats.reads = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("Writes=") {
            stats.writes = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("Blocks=") {
            stats.blocks = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("Errors=") {
            stats.errors = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("Ioctls=") {
            stats.ioctls = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("BufferBytes=") {
            stats.current_buffer_bytes = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("Sessions(Total)=") {
            stats.total_sessions = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("Sessions(Active)=") {
            stats.active_sessions = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("BytesProcessed=") {
            stats.bytes_processed = v.trim().parse().unwrap();
        }
    }

    stats
}

pub fn run_dashboard(interval: Duration) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Keep history for last N points
    const MAX_POINTS: usize = 50;
    let mut history_reads: VecDeque<(f64,f64)> = VecDeque::new();
    let mut history_writes: VecDeque<(f64,f64)> = VecDeque::new();
    let mut history_ioctls: VecDeque<(f64,f64)> = VecDeque::new();
    let mut history_bytes: VecDeque<(f64,f64)> = VecDeque::new();
    let mut x = 0.0;

    let mut prev = read_stats();

    loop {
        let current = read_stats();

        // Compute rates
        let reads_sec = (current.reads - prev.reads) as f64;
        let writes_sec = (current.writes - prev.writes) as f64;
        let ioctls_sec = (current.ioctls - prev.ioctls) as f64;
        let bytes_sec = (current.bytes_processed - prev.bytes_processed) as f64;
        let errors_sec = (current.errors - prev.errors) as f64;

        // Push to history
        for (deque, value) in [
            (&mut history_reads, reads_sec),
            (&mut history_writes, writes_sec),
            (&mut history_ioctls, ioctls_sec),
            (&mut history_bytes, bytes_sec),
        ] {
            deque.push_back((x, value));
            if deque.len() > MAX_POINTS {
                deque.pop_front();
            }
        }

        // Layout: upper = chart, lower = raw stats
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(60), // chart
                    Constraint::Percentage(40), // raw stats
                ].as_ref())
                .split(size);

            // Graph of rates
            let chart = Chart::new(vec![
                Dataset::default()
                    .name("Reads/sec")
                    .marker(ratatui::symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Green))
                    .data(&history_reads.iter().copied().collect::<Vec<_>>()),
                Dataset::default()
                    .name("Writes/sec")
                    .marker(ratatui::symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Blue))
                    .data(&history_writes.iter().copied().collect::<Vec<_>>()),
                Dataset::default()
                    .name("IOCTLs/sec")
                    .marker(ratatui::symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Yellow))
                    .data(&history_ioctls.iter().copied().collect::<Vec<_>>()),
                Dataset::default()
                    .name("Bytes/sec")
                    .marker(ratatui::symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Magenta))
                    .data(&history_bytes.iter().copied().collect::<Vec<_>>()),
            ])
            .block(Block::default().title("I/O Rates").borders(Borders::ALL))
            .x_axis(Axis::default().title("Time").bounds([x - MAX_POINTS as f64, x]))
            .y_axis(Axis::default().title("Rate").bounds([0.0, reads_sec.max(writes_sec).max(ioctls_sec).max(bytes_sec) * 1.2]));

            f.render_widget(chart, chunks[0]);

            // Raw stats
            let text = format!(
                "Raw Stats:\n\
                 Reads: {}\n\
                 Writes: {}\n\
                 IOCTLs: {}\n\
                 Blocks: {}\n\
                 Bytes Processed: {}\n\
                 Errors: {}\n\
                 Current Buffer: {}\n\
                 Active Sessions: {}\n\
                 Total Sessions: {}",
                current.reads,
                current.writes,
                current.ioctls,
                current.blocks,
                current.bytes_processed,
                current.errors,
                current.current_buffer_bytes,
                current.active_sessions,
                current.total_sessions
            );

            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Raw Stats"));

            f.render_widget(paragraph, chunks[1]);
        })?;

        prev = current;
        x += 1.0;
        std::thread::sleep(interval);
    }
}
