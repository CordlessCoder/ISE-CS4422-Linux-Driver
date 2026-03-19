use crate::stats::fetch_stats::ChaChaInstant;
use crossterm::event;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph},
};
use sizef::IntoSize;
use std::{
    io,
    time::{Duration, Instant},
};

mod fetch_stats;

pub fn run_dashboard(interval: Duration) -> Result<(), io::Error> {
    // Keep history for last N points
    const MAX_POINTS: usize = 50;
    let mut history_reads: Vec<(f64, f64)> = Vec::new();
    let mut history_writes: Vec<(f64, f64)> = Vec::new();
    let mut history_ioctls: Vec<(f64, f64)> = Vec::new();
    let mut history_bytes: Vec<(f64, f64)> = Vec::new();
    let mut history_blocks: Vec<(f64, f64)> = Vec::new();
    let mut history_sessions: Vec<(f64, f64)> = Vec::new();
    let mut history_errors: Vec<(f64, f64)> = Vec::new();
    let mut time = 0.0;

    ratatui::run(|terminal| {
        let mut next_frame = Instant::now().checked_add(interval).unwrap();
        'stats: loop {
            let current = fetch_stats::ChaChaSample::fetch().unwrap();
            let diff = current.diff_with_last();

            // Push to history
            let compute_per_second = |val| val as f64 / (diff.over.as_secs_f64());
            for (queue, value) in [
                (&mut history_reads, diff.reads),
                (&mut history_writes, diff.writes),
                (&mut history_ioctls, diff.ioctls),
                (&mut history_bytes, diff.bytes),
                (&mut history_blocks, diff.blocks * 64),
                (&mut history_errors, diff.errors),
            ] {
                if queue.len() >= MAX_POINTS {
                    queue.remove(0);
                }
                queue.push((time, compute_per_second(value)));
            }
            if history_sessions.len() >= MAX_POINTS {
                history_sessions.remove(0);
            }
            history_sessions.push((time, current.data.active_sessions as f64));

            // Layout: upper = chart, lower = raw stats
            terminal.draw(|f| {
                let area = f.area();
                let row_count = 2;
                let col_count = 2;
                let col_constraints = vec![Constraint::Ratio(1, row_count); row_count as usize];
                let row_constraints = vec![Constraint::Ratio(1, col_count); col_count as usize];
                let horizontal = Layout::horizontal(col_constraints);
                let vertical = Layout::vertical(row_constraints);
                let cells: Vec<_> = area
                    .layout_vec(&vertical)
                    .into_iter()
                    .flat_map(|row| row.layout_vec(&horizontal))
                    .collect();

                // Graph of rates
                let max_throughput = history_bytes
                    .iter()
                    .map(|&(_, b)| b)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or_default();
                let max_io = history_reads
                    .iter()
                    .chain(&history_writes)
                    .chain(&history_ioctls)
                    .map(|&(_, b)| b)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or_default();
                let max_events = history_sessions
                    .iter()
                    .chain(&history_errors)
                    .map(|&(_, b)| b)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or_default();
                let chart_throughput = Chart::new(vec![
                    Dataset::default()
                        .name("Bytes/sec")
                        .marker(ratatui::symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Blue))
                        .data(&history_bytes),
                    Dataset::default()
                        .name("Blocks/sec")
                        .marker(ratatui::symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Cyan))
                        .data(&history_blocks),
                ])
                .block(Block::default().title("Throughput").borders(Borders::ALL))
                .legend_position(Some(ratatui::widgets::LegendPosition::BottomLeft))
                .hidden_legend_constraints((Constraint::Min(1), Constraint::Min(1)))
                .x_axis(
                    Axis::default()
                        .title("Time")
                        .bounds([time - MAX_POINTS as f64 * interval.as_secs_f64(), time]),
                )
                .y_axis(
                    Axis::default()
                        .bounds([0.0, max_throughput.max(1.0)])
                        .labels([
                            "0.0B".to_string(),
                            format!("{:.0}", max_throughput.into_decimalsize()),
                        ]),
                );
                f.render_widget(chart_throughput, cells[1]);

                let chart_ops = Chart::new(vec![
                    Dataset::default()
                        .name("Reads/sec")
                        .marker(ratatui::symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Red))
                        .data(&history_reads),
                    Dataset::default()
                        .name("Writes/sec")
                        .marker(ratatui::symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Blue))
                        .data(&history_writes),
                    Dataset::default()
                        .name("IOCTLs/sec")
                        .marker(ratatui::symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Yellow))
                        .data(&history_ioctls),
                ])
                .block(Block::default().title("Operations").borders(Borders::ALL))
                .legend_position(Some(ratatui::widgets::LegendPosition::BottomLeft))
                .hidden_legend_constraints((Constraint::Min(1), Constraint::Min(1)))
                .x_axis(
                    Axis::default()
                        .title("Time")
                        .bounds([time - MAX_POINTS as f64 * interval.as_secs_f64(), time]),
                )
                .y_axis(
                    Axis::default()
                        .title("Rate")
                        .bounds([0.0, max_io.max(1.0)])
                        .labels(["0.0".to_string(), format!("{:.0}", max_io)]),
                );
                f.render_widget(chart_ops, cells[2]);

                let chart_sessions = Chart::new(vec![
                    Dataset::default()
                        .name("Errors/sec")
                        .marker(ratatui::symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Red))
                        .data(&history_errors),
                    Dataset::default()
                        .name("Active sessions")
                        .marker(ratatui::symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Blue))
                        .data(&history_sessions),
                ])
                .block(Block::default().title("Other Events").borders(Borders::ALL))
                .legend_position(Some(ratatui::widgets::LegendPosition::BottomLeft))
                .hidden_legend_constraints((Constraint::Min(1), Constraint::Min(1)))
                .x_axis(
                    Axis::default()
                        .title("Time")
                        .bounds([time - MAX_POINTS as f64 * interval.as_secs_f64(), time]),
                )
                .y_axis(
                    Axis::default()
                        .title("Rate")
                        .bounds([0.0, max_events.max(1.0)])
                        .labels(["0.0".to_string(), format!("{:.0}", max_events)]),
                );
                f.render_widget(chart_sessions, cells[3]);

                // Raw stats
                let text = {
                    let ChaChaInstant {
                        total_sessions,
                        active_sessions,
                        bytes,
                        reads,
                        writes,
                        ioctls,
                        blocks,
                        buffered_bytes,
                        errors,
                    } = current.data;
                    format!(
                        "Reads:           {reads}\n\
                         Writes:          {writes}\n\
                         IOCTLs:          {ioctls}\n\
                         Blocks:          {blocks}\n\
                         Bytes Processed: {bytes}\n\
                         Errors:          {errors}\n\
                         Current Buffer:  {buffered_bytes}\n\
                         Active Sessions: {active_sessions}\n\
                         Total Sessions:  {total_sessions}",
                    )
                };

                let paragraph = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title("Raw Stats"));

                f.render_widget(paragraph, cells[0]);
            })?;
            while event::poll(next_frame.duration_since(Instant::now()))? {
                let event = event::read()?;
                let event::Event::Key(key) = event else {
                    continue;
                };
                match key {
                    event::KeyEvent {
                        code: event::KeyCode::Char('q'),
                        kind: event::KeyEventKind::Press,
                        ..
                    } => {
                        break 'stats;
                    }
                    event::KeyEvent {
                        code: event::KeyCode::Char('c'),
                        kind: event::KeyEventKind::Press,
                        modifiers,
                        ..
                    } if modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break 'stats;
                    }
                    _ => (),
                }
            }
            time += interval.as_secs_f64();
            next_frame = next_frame.checked_add(interval).unwrap();
        }

        Ok(())
    })
}
