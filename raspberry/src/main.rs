use anyhow::Result;
use tui::widgets::{Block, Borders, Paragraph, List, ListItem, Wrap};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Style, Color, Modifier};
use tui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::thread;
use std::time::Duration;

use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event as CEvent, KeyCode},
};

use crate::controller::{AlarmController, VALID_BADGES};

mod arduino;
mod arduino_consts;
mod controller;


fn main() -> Result<()> {
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

            controller.poll()?;

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
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(lines[0]);

                let state_block = Block::default().title("État").borders(Borders::ALL);
                let state_par = Paragraph::new(format!("{}\n\n[ Mouv: {} | Btn: {} ]", controller.state_icon(), controller.motion_str(), controller.btn_str()))
                    .block(state_block)
                    .style(Style::default().fg(Color::Yellow))
                    .wrap(Wrap { trim: true });
                f.render_widget(state_par, monitor_cols[0]);

                let mid_block = Block::default().title("Dernier Badge").borders(Borders::ALL);
                let last_rfid = controller.last_rfid().unwrap_or("Aucun");
                let mid_par = Paragraph::new(format!("{}", last_rfid))
                    .block(mid_block)
                    .style(Style::default().fg(Color::White));
                f.render_widget(mid_par, monitor_cols[1]);


                let history_block = Block::default().title("Historique").borders(Borders::ALL);
                let items: Vec<ListItem> = controller.last_messages().iter()
                    .map(|m| ListItem::new(m.clone()).style(Style::default()))
                    .collect();
                let list = List::new(items).block(history_block).highlight_style(Style::default().add_modifier(Modifier::BOLD));
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
