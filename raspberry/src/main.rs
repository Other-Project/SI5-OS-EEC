use anyhow::Result;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io;
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::controller::AlarmController;
use crate::menu::Menu;
use crate::tui_logger::{init_logger, LOG_BUFFER};

mod arduino;
mod arduino_consts;
mod badges;
mod controller;
mod lcd;
mod menu;
mod tui_logger;

fn main() -> Result<()> {
    init_logger().ok();

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut controller = AlarmController::new()?;
    let mut menu = Menu::new();

    loop {
        if event::poll(Duration::from_millis(0))? {
            if let CEvent::Key(key) = event::read()? {
                if let Ok(Some(action)) = menu.handle_key(key.code, &mut controller) {
                    if action == "quit" {
                        break;
                    }
                }
            }
        }

        controller
            .poll()
            .unwrap_or_else(|e| log::error!("Controller error: {}", e));
        menu.poll(&mut controller)
            .unwrap_or_else(|e| log::error!("Menu error: {}", e));

        terminal.draw(|f| render_ui(f, &menu, &controller))?;
        thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

fn render_ui(f: &mut Frame, menu: &Menu, controller: &AlarmController) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.area());

    render_status_bar(f, controller, chunks[0]);
    render_main_content(f, menu, controller, chunks[1]);
}

fn render_status_bar(f: &mut Frame, controller: &AlarmController, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let status_items = [
        (" State ", controller.state_icon(), Color::Yellow),
        (" Motion ", controller.motion_str(), Color::White),
        (" Button ", controller.btn_str(), Color::White),
        (
            " Last Badge ",
            controller.last_rfid().unwrap_or("None"),
            Color::White,
        ),
    ];

    for (i, (title, content, color)) in status_items.iter().enumerate() {
        let block = Block::default()
            .title(*title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let par = Paragraph::new(*content)
            .block(block)
            .style(Style::default().fg(*color))
            .alignment(Alignment::Center);
        f.render_widget(par, chunks[i]);
    }
}

fn render_main_content(
    f: &mut Frame,
    menu: &Menu,
    _controller: &AlarmController,
    area: ratatui::layout::Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let tab_index = if menu.main_tab == menu::MainTab::Logs {
        0
    } else {
        1
    };
    let tabs = Tabs::new(vec!["Logs", "Badges"])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .select(tab_index)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    let help_text = if menu.main_tab == menu::MainTab::Logs {
        "[TAB] Switch Tab  [Q] Quit".to_string()
    } else {
        menu.get_bottom_help()
    };

    match menu.main_tab {
        menu::MainTab::Logs => render_logs_tab(f, chunks[1]),
        menu::MainTab::Badges => menu.render(f, chunks[1]),
    }

    render_help_bar(f, help_text, chunks[2]);
}

fn render_logs_tab(f: &mut Frame, area: ratatui::layout::Rect) {
    let logs_height = area.height.saturating_sub(2) as usize;
    let items: Vec<ListItem> = {
        let buf = LOG_BUFFER.lock().unwrap();
        buf.iter()
            .rev()
            .take(logs_height)
            .map(|m| ListItem::new(m.clone()))
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, area);
}

fn render_help_bar(f: &mut Frame, help_text: String, area: ratatui::layout::Rect) {
    let help_par = Paragraph::new(Line::from(vec![Span::styled(
        help_text,
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(help_par, area);
}
