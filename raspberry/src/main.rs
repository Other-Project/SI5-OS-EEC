use anyhow::Result;
use std::io;
use std::thread;
use std::time::Duration;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::{backend::CrosstermBackend, Terminal};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::controller::AlarmController;

// --- Add imports for logging ---
use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};
use std::sync::{Mutex, Arc};
use lazy_static::lazy_static;

// --- Add static buffer for log messages ---
lazy_static! {
    static ref LOG_BUFFER: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

// --- Logger implementation ---
struct TuiLogger;

impl log::Log for TuiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info // Change as needed
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("[{}] {}", record.level(), record.args());
            let mut buf = LOG_BUFFER.lock().unwrap();
            buf.push(msg);
            // Limit buffer size
            if buf.len() > 100 {
                buf.remove(0);
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: TuiLogger = TuiLogger;

// --- Logger initialization function ---
fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
}

mod arduino;
mod arduino_consts;
mod controller;

fn main() -> Result<()> {
    // --- Initialize logger ---
    init_logger().ok();

    // setup terminal
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut controller = AlarmController::new()?;

    let res = (|| -> Result<()> {
        loop {
            // check keyboard events (non-blocking)
            if event::poll(Duration::from_millis(0))? {
                if let CEvent::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }

            if let Err(e) = controller.poll() {
                log::error!("{}", e);
            }

            terminal.draw(|f| {
                let size = f.size();
                let lines = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                    .split(size);

                // Left: main state
                let monitor_cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(25),
                            Constraint::Percentage(25),
                            Constraint::Percentage(25),
                            Constraint::Percentage(25),
                        ]
                        .as_ref(),
                    )
                    .split(lines[0]);

                let state_block = Block::default().title("State").borders(Borders::ALL);
                let state_par = Paragraph::new(controller.state_icon())
                    .block(state_block)
                    .style(Style::default().fg(Color::Yellow));
                f.render_widget(state_par, monitor_cols[0]);

                let motion_block = Block::default().title("Motion").borders(Borders::ALL);
                let motion_par = Paragraph::new(controller.motion_str())
                    .block(motion_block)
                    .style(Style::default().fg(Color::White));
                f.render_widget(motion_par, monitor_cols[1]);

                let button_block = Block::default().title("Button").borders(Borders::ALL);
                let button_par = Paragraph::new(controller.btn_str())
                    .block(button_block)
                    .style(Style::default().fg(Color::White));
                f.render_widget(button_par, monitor_cols[2]);

                let mid_block = Block::default().title("Last Badge").borders(Borders::ALL);
                let last_rfid = controller.last_rfid().unwrap_or("None");
                let mid_par = Paragraph::new(format!("{}", last_rfid))
                    .block(mid_block)
                    .style(Style::default().fg(Color::White));
                f.render_widget(mid_par, monitor_cols[3]);

                let logs_block = Block::default().title("Logs").borders(Borders::ALL);
                let logs_height = lines[1].height.saturating_sub(2) as usize;
                let log_lines: Vec<String> = {
                    let buf = LOG_BUFFER.lock().unwrap();
                    buf.iter().rev().take(logs_height).cloned().collect()
                };
                let items: Vec<ListItem> = log_lines
                    .into_iter()
                    .map(|m| ListItem::new(m).style(Style::default()))
                    .collect();
                let list = List::new(items)
                    .block(logs_block)
                    .highlight_style(Style::default().add_modifier(Modifier::BOLD));
                f.render_widget(list, lines[1]);
            })?;

            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    })();

    // restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res?;
    Ok(())
}
