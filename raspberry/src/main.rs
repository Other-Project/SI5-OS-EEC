use anyhow::Result;
use clap::Parser;
use log::info;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::arduino_consts::SecurityState;
use crate::config_menu::ConfigMenu;
use crate::controller::AlarmController;
use crate::lcd_controller::LcdController;
use crate::menu::Menu;
use crate::tui_logger::init_logger;
use crate::{badge_menu::BadgeMenu, log_menu::LogMenu};

mod arduino;
mod arduino_consts;
mod badge_menu;
mod badges;
mod config_menu;
mod controller;
mod lcd;
mod lcd_controller;
mod log_menu;
mod menu;
mod tui_logger;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable TUI mode
    #[arg(short, long)]
    tui: bool,
}

fn main() -> Result<()> {
    let cli = Args::parse();

    let mut terminal = if cli.tui {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        init_logger().ok();
        Some(terminal)
    } else {
        env_logger::builder()
            .is_test(false)
            .filter_level(log::LevelFilter::Info)
            .format(|buf, record| {
                let now: chrono::DateTime<chrono::Local> = chrono::Local::now();
                writeln!(
                    buf,
                    "[{}] [{}] {}",
                    now.format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .init();
        info!("Starting in daemon mode");
        None
    };

    let res = run_app(&mut terminal);

    if let Some(term) = terminal.as_mut() {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        term.show_cursor()?;
    }

    res
}

fn run_app(terminal: &mut Option<Terminal<CrosstermBackend<io::Stdout>>>) -> Result<()> {
    let controller = Rc::new(RefCell::new(AlarmController::new()?));
    let mut lcd_controller = LcdController::new(controller.clone())?;
    let mut menu = if terminal.is_some() {
        Some(Menu::new(vec![
            Box::new(LogMenu::new()),
            Box::new(BadgeMenu::new(controller.clone())),
            Box::new(ConfigMenu::new(controller.clone())),
        ]))
    } else {
        None
    };

    loop {
        if let Some(ref mut menu_instance) = menu {
            if event::poll(Duration::from_millis(0))? {
                if let CEvent::Key(key) = event::read()? {
                    match menu_instance.handle_key(key.code) {
                        Ok(should_quit) => {
                            if should_quit {
                                break;
                            }
                        }
                        Err(e) => {
                            log::error!("Menu key handling error: {}", e);
                        }
                    }
                }
            }
        }

        controller
            .borrow_mut()
            .poll()
            .unwrap_or_else(|e| log::error!("Controller error: {}", e));

        lcd_controller
            .poll()
            .unwrap_or_else(|e| log::error!("LCD controller error: {}", e));

        if let Some(ref mut menu_instance) = menu {
            menu_instance
                .poll()
                .unwrap_or_else(|e| log::error!("Menu error: {}", e));
        }

        if let Some(ref mut term) = terminal {
            if let Some(ref menu_instance) = menu {
                term.draw(|f| render_ui(f, menu_instance, &controller.borrow()))?;
            }
        }

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
        (" State ", match controller.state() {
            SecurityState::Disarmed => "🟢 DISARMED",
            SecurityState::Armed => "🛑 ARMED",
            SecurityState::Triggered => "🚨 ALARM",
        }, Color::White),
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
            .border_style(Style::default().fg(Color::DarkGray));
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

    let tabs = Tabs::new(menu.tabs())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .select(menu.current_tab)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    let help_text = menu.get_bottom_help();
    menu.render(f, chunks[1]);

    render_help_bar(f, help_text, chunks[2]);
}

fn render_help_bar(f: &mut Frame, help_text: String, area: ratatui::layout::Rect) {
    let help_par = Paragraph::new(Line::from(vec![Span::styled(
        help_text,
        Style::default().fg(Color::DarkGray),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(help_par, area);
}
