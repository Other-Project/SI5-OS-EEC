use std::cell::RefCell;
use std::rc::Rc;

use crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::controller::AlarmController;
use crate::menu::TuiMenuTrait;

pub struct ConfigMenu {
    pub controller: Rc<RefCell<AlarmController>>,
    pub distance: f32,
}

impl ConfigMenu {
    pub fn new(controller: Rc<RefCell<AlarmController>>) -> Self {
        let distance = controller
            .borrow_mut()
            .get_ultrasonic_distance()
            .unwrap_or(4.0);
        Self {
            controller,
            distance,
        }
    }
}

impl TuiMenuTrait for ConfigMenu {
    fn name(&self) -> &'static str {
        "Config"
    }

    fn poll(&mut self) -> anyhow::Result<()> {
        self.distance = self
            .controller
            .borrow_mut()
            .get_ultrasonic_distance()
            .unwrap_or(4.0);
        Ok(())
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let block = Block::default()
            .title(" Configuration ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(area);

        let distance_paragraph = Paragraph::new(Line::from(Span::raw(format!(
            "Ultrasonic Distance: {:.1} cm",
            self.distance
        ))))
        .alignment(Alignment::Left);
        f.render_widget(distance_paragraph, chunks[0]);
    }
}
