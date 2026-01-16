use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;

pub struct Menu {
    pub current_tab: usize,
    pub tabs: Vec<Box<dyn TuiMenuTrait>>,
}

impl Menu {
    pub fn new(tabs: Vec<Box<dyn TuiMenuTrait>>) -> Self {
        Self {
            current_tab: 0,
            tabs,
        }
    }

    fn get_current_tab_mut(&mut self) -> &mut dyn TuiMenuTrait {
        self.tabs[self.current_tab].as_mut()
    }
    fn get_current_tab_ref(&self) -> &dyn TuiMenuTrait {
        self.tabs[self.current_tab].as_ref()
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        let handled = self.get_current_tab_mut().handle_key(key)?;
        if handled {
            return Ok(false);
        }

        match key {
            KeyCode::Tab => {
                self.current_tab = (self.current_tab + 1) % self.tabs.len();
                self.get_current_tab_mut().reset()?;
                return Ok(false);
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
            _ => Ok(false),
        }
    }

    pub fn poll(&mut self) -> Result<()> {
        self.get_current_tab_mut().poll()?;
        Ok(())
    }

    pub fn get_bottom_help(&self) -> String {
        self.get_current_tab_ref().key_help().unwrap_or(" [TAB] Switch Tab  [Q] Quit ".to_string())
    }

    pub fn render(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        self.get_current_tab_ref().render(f, area);
    }
}

pub trait TuiMenuTrait {
    fn reset(&mut self) -> Result<()>;
    fn render(&self, f: &mut Frame, area: ratatui::layout::Rect);
    fn poll(&mut self) -> Result<()>;
    fn key_help(&self) -> Option<String>;
    fn handle_key(&mut self, key: KeyCode) -> Result<bool>;
}
