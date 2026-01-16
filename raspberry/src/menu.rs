use anyhow::Result;
use chrono::{Duration, Utc};
use crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::controller::AlarmController;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BadgeTab {
    ListBadges,
    AddBadge,
    RemoveBadge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MainTab {
    Logs,
    Badges,
}

#[derive(Debug, Clone)]
pub struct BadgeField {
    pub uid: String,
    pub name: String,
    pub expiry_days: String,
    pub selected_field: usize, // 0 = name, 1 = expiry_days
}

pub struct Menu {
    pub main_tab: MainTab,
    pub badge_tab: BadgeTab,
    pub selected_item: usize,
    pub max_items: usize,
    pub last_badge_uid: Option<String>,
    pub message: Option<String>,
    pub badges_list: Vec<(String, String)>, // (uid, name)
    pub badge_statuses: Vec<bool>, // enabled status for each badge
    pub badge_field: Option<BadgeField>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            main_tab: MainTab::Logs,
            badge_tab: BadgeTab::ListBadges,
            selected_item: 0,
            max_items: 0,
            last_badge_uid: None,
            message: None,
            badges_list: Vec::new(),
            badge_statuses: Vec::new(),
            badge_field: None,
        }
    }

    fn load_badges_with_status(&mut self, controller: &AlarmController) -> Result<()> {
        self.badges_list = controller.get_all_badges()?;
        self.badge_statuses = self.badges_list
            .iter()
            .map(|(uid, _)| controller.badge_manager().is_valid_badge(uid).unwrap_or(false))
            .collect();
        self.max_items = self.badges_list.len().max(1);
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyCode, controller: &mut AlarmController) -> Result<Option<String>> {
        match key {
            KeyCode::Tab if self.badge_field.is_none() => {
                self.main_tab = match self.main_tab {
                    MainTab::Logs => {
                        self.badge_tab = BadgeTab::ListBadges;
                        self.load_badges_with_status(controller)?;
                        self.selected_item = 0;
                        MainTab::Badges
                    },
                    MainTab::Badges => MainTab::Logs,
                };
                return Ok(None);
            }
            KeyCode::Char('q') | KeyCode::Char('Q') if self.badge_field.is_none() => {
                return Ok(Some("quit".to_string()));
            }
            _ => {}
        }

        if self.main_tab != MainTab::Badges {
            return Ok(None);
        }

        if let Some(field) = self.badge_field.clone() {
            return self.handle_badge_field_input(key, &field, controller);
        }

        match self.badge_tab {
            BadgeTab::AddBadge => self.handle_add_badge_keys(key, controller),
            BadgeTab::RemoveBadge => self.handle_remove_badge_keys(key, controller),
            BadgeTab::ListBadges => self.handle_list_badges(key, controller),
        }
    }

    pub fn poll(&mut self, controller: &mut AlarmController) -> Result<()> {
        // Check for scanned badges during Add/Remove operations
        match self.badge_tab {
            BadgeTab::AddBadge => self.check_add_badge_scan(controller)?,
            BadgeTab::RemoveBadge => self.check_remove_badge_scan(controller)?,
            _ => {}
        }
        Ok(())
    }

    fn check_add_badge_scan(&mut self, controller: &mut AlarmController) -> Result<()> {
        // Check if a badge was scanned
        if let Some(scanned_uid) = controller.get_last_rfid() {
            if self.last_badge_uid.as_deref() != Some(scanned_uid) {
                self.last_badge_uid = Some(scanned_uid.to_string());
                
                // Start field input for badge name
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

    fn check_remove_badge_scan(&mut self, controller: &mut AlarmController) -> Result<()> {
        // Check if a badge was scanned
        if let Some(scanned_uid) = controller.get_last_rfid() {
            if self.last_badge_uid.as_deref() != Some(scanned_uid) {
                self.last_badge_uid = Some(scanned_uid.to_string());
                
                // Try to remove badge
                if let Ok(Some(badge)) = controller.badge_manager().get_badge(scanned_uid) {
                    controller.badge_manager().remove_badge(scanned_uid)?;
                    self.message = Some(format!("Badge removed: {}", badge.name));
                } else {
                    self.message = Some("Badge not found in database".to_string());
                }
            }
        }
        Ok(())
    }

    fn handle_add_badge_keys(&mut self, key: KeyCode, controller: &mut AlarmController) -> Result<Option<String>> {
        match key {
            KeyCode::Esc => {
                self.badge_tab = BadgeTab::ListBadges;
                self.badge_field = None;
                self.message = None;
                self.load_badges_with_status(controller)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_remove_badge_keys(&mut self, key: KeyCode, controller: &mut AlarmController) -> Result<Option<String>> {
        match key {
            KeyCode::Esc => {
                self.badge_tab = BadgeTab::ListBadges;
                self.last_badge_uid = None;
                self.message = None;
                self.load_badges_with_status(controller)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_badge_field_input(&mut self, key: KeyCode, field: &BadgeField, controller: &mut AlarmController) -> Result<Option<String>> {
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
                return self.confirm_add_badge(field, controller);
            }
            KeyCode::Esc => {
                self.cancel_badge_operation(controller)?;
            }
            _ => {}
        }
        Ok(None)
    }

    fn confirm_add_badge(&mut self, field: &BadgeField, controller: &mut AlarmController) -> Result<Option<String>> {
        let expiry_date = if !field.expiry_days.is_empty() {
            field.expiry_days.parse::<i64>().ok()
                .and_then(|days| Utc::now().checked_add_signed(Duration::days(days)))
        } else {
            None
        };
        
        controller.badge_manager().add_badge_with_expiry(&field.uid, &field.name, expiry_date)?;
        self.message = Some(format!("Badge added: {} ({})", field.name, field.uid));
        self.cancel_badge_operation(controller)?;
        Ok(None)
    }

    fn cancel_badge_operation(&mut self, controller: &AlarmController) -> Result<()> {
        self.badge_field = None;
        self.badge_tab = BadgeTab::ListBadges;
        self.message = None;
        self.load_badges_with_status(controller)
    }

    fn handle_list_badges(&mut self, key: KeyCode, controller: &mut AlarmController) -> Result<Option<String>> {
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
                // Add badge - start waiting for scan
                self.badge_tab = BadgeTab::AddBadge;
                self.message = Some("Place badge near scanner...".to_string());
                Ok(None)
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Delete selected badge
                if let Some((uid, _name)) = self.badges_list.get(self.selected_item) {
                    controller.badge_manager().remove_badge(uid)?;
                    self.load_badges_with_status(controller)?;
                    if self.selected_item >= self.max_items {
                        self.selected_item = self.max_items.saturating_sub(1);
                    }
                }
                Ok(None)
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Remove badge by scan
                self.badge_tab = BadgeTab::RemoveBadge;
                self.message = Some("Place badge near scanner to remove...".to_string());
                Ok(None)
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // Toggle enable/disable
                if let Some((uid, _name)) = self.badges_list.get(self.selected_item) {
                    let is_enabled = controller.badge_manager().is_valid_badge(uid).unwrap_or(false);
                    if is_enabled {
                        controller.badge_manager().disable_badge(uid)?;
                    } else {
                        controller.badge_manager().enable_badge(uid)?;
                    }
                    self.load_badges_with_status(controller)?;
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    pub fn get_bottom_help(&self) -> &str {
        match self.badge_tab {
            BadgeTab::ListBadges => "[A] Add  [D] Delete  [R] Remove by Scan  [E] Toggle  [↑/↓] Navigate  [TAB] Switch Tab  [Q] Quit",
            BadgeTab::AddBadge if self.badge_field.is_some() => "[↑/↓] Navigate  [ENTER] Confirm  [ESC] Cancel",
            BadgeTab::AddBadge | BadgeTab::RemoveBadge => "[ESC] Back",
        }
    }

    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        match self.badge_tab {
            BadgeTab::ListBadges => self.render_list_badges(f, area),
            BadgeTab::AddBadge => self.render_add_badge(f, area),
            BadgeTab::RemoveBadge => self.render_remove_badge(f, area),
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
        let msg = self.message.as_deref().unwrap_or("Place badge near scanner...");
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(msg, Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("Waiting for scan...", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("Badge will be removed immediately", Style::default().fg(Color::Red))),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .title(" Remove Badge ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)))
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
            .border_style(Style::default().fg(Color::Cyan));
        f.render_widget(block, area);

        // UID
        let uid_text = Paragraph::new(Line::from(vec![
            Span::styled("UID: ", Style::default().fg(Color::Gray)),
            Span::styled(&field.uid, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));
        f.render_widget(uid_text, chunks[0]);

        // Name field
        self.render_field(f, chunks[2], "Name", &field.name, field.selected_field == 0, None);

        // Expiry field
        let expiry_display = if field.expiry_days.is_empty() { 
            "(leave empty for no expiry)".to_string() 
        } else if field.selected_field == 1 { 
            format!("{}_", field.expiry_days) 
        } else { 
            format!("{} days", field.expiry_days) 
        };
        self.render_field(f, chunks[3], "Expiry (in days)", &expiry_display, field.selected_field == 1, None);
    }

    fn render_field(&self, f: &mut Frame, area: ratatui::layout::Rect, label: &str, value: &str, is_selected: bool, display_suffix: Option<&str>) {
        let style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
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

    fn render_waiting_screen(&self, f: &mut Frame, area: ratatui::layout::Rect, title: &str, message: &str) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(message, Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("Waiting for scan...", Style::default().fg(Color::DarkGray))),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)))
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
            Line::from(Span::styled("No badges in database", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("Press [A] to add a badge", Style::default().fg(Color::Yellow))),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .title(" Badges ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn render_badges_list(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let items: Vec<ListItem> = self.badges_list
            .iter()
            .enumerate()
            .map(|(idx, (uid, name))| {
                let is_selected = idx == self.selected_item;
                let style = if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                
                let is_enabled = self.badge_statuses.get(idx).copied().unwrap_or(false);
                let status = if is_enabled {
                    Span::styled(" ✓", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled(" ✗", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                };
                
                ListItem::new(Line::from(vec![
                    Span::raw(if is_selected { "▶ " } else { "  " }),
                    status,
                    Span::raw(" "),
                    Span::styled(name, style.clone()),
                    Span::raw(" "),
                    Span::styled(format!("({})", uid), Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title(format!(" Badges ({}) ", self.badges_list.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(list, area);
    }
}

