use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;

use crate::badge_menu::BadgeMenu;
use crate::controller::AlarmController;

#[derive(Debug, Clone, PartialEq)]
pub enum MainTab {
    Logs,
    Badges,
}

pub struct Menu {
    pub main_tab: MainTab,
    pub badge_menu: BadgeMenu,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            main_tab: MainTab::Logs,
            badge_menu: BadgeMenu::new(),
        }
    }

    pub fn handle_key(
        &mut self,
        key: KeyCode,
        controller: &mut AlarmController,
    ) -> Result<Option<String>> {
        match key {
            KeyCode::Tab if self.badge_menu.badge_field.is_none() => {
                self.main_tab = match self.main_tab {
                    MainTab::Logs => {
                        self.badge_menu.reset(controller)?;
                        MainTab::Badges
                    }
                    MainTab::Badges => MainTab::Logs,
                };
                return Ok(None);
            }
            KeyCode::Char('q') | KeyCode::Char('Q') if self.badge_menu.badge_field.is_none() => {
                return Ok(Some("quit".to_string()));
            }
            _ => {}
        }

        if self.main_tab == MainTab::Badges {
            return self.badge_menu.handle_key(key, controller);
        }

        Ok(None)
    }

    pub fn poll(&mut self, controller: &mut AlarmController) -> Result<()> {
        if self.main_tab == MainTab::Badges {
            self.badge_menu.poll(controller)?;
        }
        Ok(())
    }

    pub fn get_bottom_help(&self) -> String {
        if self.main_tab == MainTab::Badges {
            self.badge_menu.get_bottom_help()
        } else {
            "[TAB] Switch Tab  [Q] Quit".to_string()
        }
    }

    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if self.main_tab == MainTab::Badges {
            self.badge_menu.render(f, area);
        } else {
            // ...existing code for rendering logs or other tabs...
        }
    }
}
