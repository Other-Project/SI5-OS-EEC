use anyhow::Result;
use chrono::{Duration, Utc};
use crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;
use std::cell::RefCell;
use std::rc::Rc;

use crate::badges::Badge;
use crate::controller::AlarmController;
use crate::menu::TuiMenuTrait;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BadgeTab {
    ListBadges,
    AddBadge,
    RemoveBadge,
}

#[derive(Debug, Clone)]
pub struct BadgeField {
    pub uid: String,
    pub name: String,
    pub expiry_days: String,
    pub selected_field: usize,
}

pub struct BadgeMenu {
    pub controller: Rc<RefCell<AlarmController>>,
    pub badge_tab: BadgeTab,
    pub selected_item: usize,
    pub max_items: usize,
    pub last_badge_uid: Option<String>,
    pub message: Option<String>,
    pub badges_list: Vec<Badge>,
    pub badge_field: Option<BadgeField>,
}

impl BadgeMenu {
    pub fn new(controller: Rc<RefCell<AlarmController>>) -> Self {
        Self {
            controller,
            badge_tab: BadgeTab::ListBadges,
            selected_item: 0,
            max_items: 0,
            last_badge_uid: None,
            message: None,
            badges_list: Vec::new(),
            badge_field: None,
        }
    }

    fn load_badges_with_status(&mut self) -> Result<()> {
        self.badges_list = self.controller.borrow_mut().badge_manager().get_all_badges()?;
        self.max_items = self.badges_list.len().max(1);
        Ok(())
    }

    fn check_add_badge_scan(&mut self) -> Result<()> {
        let controller = self.controller.borrow();
        if let Some(scanned_uid) = controller.get_last_rfid() {
            if self.last_badge_uid.as_deref() != Some(scanned_uid) {
                self.last_badge_uid = Some(scanned_uid.to_string());
                self.badge_field = Some(BadgeField {
                    uid: scanned_uid.to_string(),
                    name: format!("Badge {}", scanned_uid.chars().take(4).collect::<String>()),
                    expiry_days: String::new(),
                    selected_field: 0,
                });
                self.message = None;
            }
        }
        Ok(())
    }

    fn check_remove_badge_scan(&mut self) -> Result<()> {
        let controller = self.controller.borrow();
        if let Some(scanned_uid) = controller.get_last_rfid() {
            if self.last_badge_uid.as_deref() != Some(scanned_uid) {
                self.last_badge_uid = Some(scanned_uid.to_string());
                let controller_mut = self.controller.borrow_mut();
                if let Ok(Some(badge)) = controller_mut.badge_manager().get_badge(scanned_uid) {
                    controller_mut.badge_manager().remove_badge(scanned_uid)?;
                    self.message = Some(format!("Badge removed: {}", badge.name));
                } else {
                    self.message = Some("Badge not found in database".to_string());
                }
            }
        }
        Ok(())
    }

    fn handle_add_badge_keys(
        &mut self,
        key: KeyCode,
    ) -> Result<Option<String>> {
        match key {
            KeyCode::Esc => {
                self.badge_tab = BadgeTab::ListBadges;
                self.badge_field = None;
                self.message = None;
                self.load_badges_with_status()?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_remove_badge_keys(
        &mut self,
        key: KeyCode,
    ) -> Result<Option<String>> {
        match key {
            KeyCode::Esc => {
                self.badge_tab = BadgeTab::ListBadges;
                self.last_badge_uid = None;
                self.message = None;
                self.load_badges_with_status()?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_badge_field_input(
        &mut self,
        key: KeyCode,
        field: &BadgeField,
    ) -> Result<Option<String>> {
        match key {
            KeyCode::Down if field.selected_field < 1 => {
                if let Some(ref mut f) = self.badge_field {
                    f.selected_field += 1;
                }
            }
            KeyCode::Up if field.selected_field > 0 => {
                if let Some(ref mut f) = self.badge_field {
                    f.selected_field -= 1;
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut f) = self.badge_field {
                    let _ = match field.selected_field {
                        0 => f.name.pop(),
                        1 => f.expiry_days.pop(),
                        _ => None,
                    };
                }
            }
            KeyCode::Char(c) => {
                if let Some(ref mut f) = self.badge_field {
                    match field.selected_field {
                        0 if f.name.len() < 30 => f.name.push(c),
                        1 if c.is_ascii_digit() && f.expiry_days.len() < 4 => f.expiry_days.push(c),
                        _ => {}
                    }
                }
            }
            KeyCode::Enter => {
                return self.confirm_add_badge(field);
            }
            KeyCode::Esc => {
                self.cancel_badge_operation()?;
            }
            _ => {}
        }
        Ok(None)
    }

    fn confirm_add_badge(
        &mut self,
        field: &BadgeField,
    ) -> Result<Option<String>> {
        let expiry_date = if !field.expiry_days.is_empty() {
            field
                .expiry_days
                .parse::<i64>()
                .ok()
                .and_then(|days| Utc::now().checked_add_signed(Duration::days(days)))
        } else {
            None
        };

        self.controller.borrow_mut()
            .badge_manager()
            .add_badge_with_expiry(&field.uid, &field.name, expiry_date)?;
        self.message = Some(format!("Badge added: {} ({})", field.name, field.uid));
        self.cancel_badge_operation()?;
        Ok(None)
    }

    fn cancel_badge_operation(&mut self) -> Result<()> {
        self.badge_field = None;
        self.badge_tab = BadgeTab::ListBadges;
        self.message = None;
        self.load_badges_with_status()
    }

    fn handle_list_badges(
        &mut self,
        key: KeyCode,
    ) -> Result<Option<String>> {
        match key {
            KeyCode::Up => {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                }
                Ok(None)
            }
            KeyCode::Down => {
                if self.selected_item < self.max_items.saturating_sub(1) {
                    self.selected_item += 1;
                }
                Ok(None)
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.badge_tab = BadgeTab::AddBadge;
                self.message = Some("Place badge near scanner...".to_string());
                Ok(None)
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(badge) = self.badges_list.get(self.selected_item) {
                    self.controller.borrow_mut().badge_manager().remove_badge(&badge.uid)?;
                    self.load_badges_with_status()?;
                    if self.selected_item >= self.max_items {
                        self.selected_item = self.max_items.saturating_sub(1);
                    }
                }
                Ok(None)
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.badge_tab = BadgeTab::RemoveBadge;
                self.message = Some("Place badge near scanner to remove...".to_string());
                Ok(None)
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(badge) = self.badges_list.get(self.selected_item) {
                    let is_enabled = self.controller.borrow()
                        .badge_manager()
                        .is_valid_badge(&badge.uid)
                        .unwrap_or(false);
                    if is_enabled {
                        self.controller.borrow_mut().badge_manager().disable_badge(&badge.uid)?;
                    } else {
                        self.controller.borrow_mut().badge_manager().enable_badge(&badge.uid)?;
                    }
                    self.load_badges_with_status()?;
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn render_add_badge(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if let Some(ref field) = self.badge_field {
            self.render_badge_form(f, area, field);
        } else {
            self.render_waiting_screen(f, area, "Add Badge", "Place badge near scanner...");
        }
    }

    fn render_remove_badge(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let msg = self
            .message
            .as_deref()
            .unwrap_or("Place badge near scanner...");
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(msg, Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled(
                "Waiting for scan...",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Badge will be removed immediately",
                Style::default().fg(Color::Red),
            )),
        ];
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Remove Badge ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn render_badge_form(&self, f: &mut Frame, area: ratatui::layout::Rect, field: &BadgeField) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);

        let block = Block::default()
            .title(" Add Badge ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(block, area);

        let uid_text = Paragraph::new(Line::from(vec![
            Span::styled("UID: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &field.uid,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        f.render_widget(uid_text, chunks[0]);

        self.render_field(
            f,
            chunks[2],
            "Name",
            &field.name,
            field.selected_field == 0,
            None,
        );

        let expiry_display = if field.expiry_days.is_empty() {
            "(leave empty for no expiry)".to_string()
        } else if field.selected_field == 1 {
            format!("{}_ days", field.expiry_days)
        } else {
            format!("{} days", field.expiry_days)
        };
        self.render_field(
            f,
            chunks[3],
            "Validity",
            &expiry_display,
            field.selected_field == 1,
            None,
        );
    }

    fn render_field(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        label: &str,
        value: &str,
        is_selected: bool,
        display_suffix: Option<&str>,
    ) {
        let style = if is_selected {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let marker = if is_selected { "▶ " } else { "  " };
        let display_value = if is_selected && display_suffix.is_none() {
            format!("{}_", value)
        } else {
            value.to_string()
        };

        let text = Paragraph::new(Line::from(vec![
            Span::raw(marker),
            Span::styled(format!("{}: ", label), Style::default().fg(Color::Gray)),
            Span::styled(display_value, style),
        ]));
        f.render_widget(text, area);
    }

    fn render_waiting_screen(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        title: &str,
        message: &str,
    ) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(message, Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled(
                "Waiting for scan...",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn render_list_badges(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if self.badges_list.is_empty() {
            self.render_empty_badges_list(f, area);
        } else {
            self.render_badges_list(f, area);
        }
    }

    fn render_empty_badges_list(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No badges in database",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press [A] to add a badge",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Badges ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn render_badges_list(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self
            .badges_list
            .iter()
            .enumerate()
            .map(|(idx, badge)| {
                let is_selected = idx == self.selected_item;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let status = if badge.enabled {
                    Span::styled(
                        " ✓",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        " ✗",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )
                };

                ListItem::new(Line::from(vec![
                    Span::raw(if is_selected { "▶ " } else { "  " }),
                    status,
                    Span::raw(" "),
                    Span::styled(&badge.name, style.clone()),
                    Span::raw(" "),
                    Span::styled(format!("({})", &badge.uid), Style::default().fg(Color::Gray)),
                    Span::raw(" "),
                    Span::styled(
                       if badge.expires_at.is_some() {
                           format!(
                               "[Expires: {}]",
                               badge
                                   .expires_at
                                   .unwrap()
                                   .format("%Y-%m-%d")
                                   .to_string()
                           )
                       } else {
                           "[No Expiry]".to_string()
                       },
                       Style::default().fg(Color::Gray)
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("[Created: {}]", badge.created_at.format("%Y-%m-%d").to_string()),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        if let Some(last_used) = badge.last_used {
                            format!("[Last Used: {}]", last_used.format("%Y-%m-%d").to_string())
                        } else {
                            "[Never Used]".to_string()
                        },
                        Style::default().fg(Color::Gray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(format!(" Badges ({}) ", self.badges_list.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(list, area);
    }
}

// Implement TuiMenuTrait for BadgeMenu
impl TuiMenuTrait for BadgeMenu {
    fn reset(&mut self) -> anyhow::Result<()> {
        self.badge_tab = BadgeTab::ListBadges;
        self.selected_item = 0;
        self.last_badge_uid = None;
        self.message = None;
        self.badge_field = None;
        self.load_badges_with_status()
    }

    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        match self.badge_tab {
            BadgeTab::ListBadges => self.render_list_badges(f, area),
            BadgeTab::AddBadge => self.render_add_badge(f, area),
            BadgeTab::RemoveBadge => self.render_remove_badge(f, area),
        }
    }

    fn poll(&mut self) -> anyhow::Result<()> {
        match self.badge_tab {
            BadgeTab::AddBadge => self.check_add_badge_scan()?,
            BadgeTab::RemoveBadge => self.check_remove_badge_scan()?,
            _ => {}
        }
        Ok(())
    }

    fn key_help(&self) -> Option<String> {
        Some(match self.badge_tab {
            BadgeTab::ListBadges => format!(
                "[A] Add  [R] Remove by Scan {} [TAB] Switch Tab  [Q] Quit",
                if self.badges_list.len() > 0 {
                    " [D] Delete Selected  [E] Toggle Selected  [↑/↓] Navigate "
                } else {
                    ""
                }
            ),
            BadgeTab::AddBadge if self.badge_field.is_some() => {
                "[↑/↓] Navigate  [ENTER] Confirm  [ESC] Cancel".to_string()
            }
            BadgeTab::AddBadge | BadgeTab::RemoveBadge => "[ESC] Back".to_string(),
        })
    }

    fn handle_key(&mut self, key: KeyCode) -> anyhow::Result<bool> {
        if let Some(field) = self.badge_field.clone() {
            self.handle_badge_field_input(key, &field)?;
            return Ok(false);
        }

        match self.badge_tab {
            BadgeTab::AddBadge => { self.handle_add_badge_keys(key)?; }
            BadgeTab::RemoveBadge => { self.handle_remove_badge_keys(key)?; }
            BadgeTab::ListBadges => { self.handle_list_badges(key)?; }
        }
        Ok(self.badge_tab != BadgeTab::ListBadges)
    }
}
