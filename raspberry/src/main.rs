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

                let history_block = Block::default().title("History").borders(Borders::ALL);
                let items: Vec<ListItem> = controller
                    .last_messages()
                    .iter()
                    .map(|m| ListItem::new(m.clone()).style(Style::default()))
                    .collect();
                let list = List::new(items)
                    .block(history_block)
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
