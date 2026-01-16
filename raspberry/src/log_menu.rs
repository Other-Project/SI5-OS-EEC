
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::menu::TuiMenuTrait;
use crate::tui_logger::LOG_BUFFER;

pub struct LogMenu {
}

impl LogMenu {
    pub fn new() -> Self {
        Self { }
    }
}

impl TuiMenuTrait for LogMenu {
    fn name(&self) -> &'static str {
        "Logs"
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
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
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(list, area);
    }
}
