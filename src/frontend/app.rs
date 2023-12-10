use crate::log::Log;

use super::{dialog::Dialog, window::Window};

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
}

impl Mode {
    pub fn is_insert(&self) -> bool {
        if let Mode::Insert = self {
            true
        } else {
            false
        }
    }

    pub fn is_normal(&self) -> bool {
        if let Mode::Normal = self {
            true
        } else {
            false
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
        }
    }
}

pub struct App {
    pub edit_windows: Vec<Window>,
    pub selected_window: usize,
    pub current_dialog: Option<Dialog>,
    pub log: Log,
    pub current_mode: Mode,
}

impl App {
    pub fn selected_window_mut(&mut self) -> Option<&'_ mut Window> {
        if self.edit_windows.is_empty() {
            None
        } else {
            Some(&mut self.edit_windows[self.selected_window])
        }
    }
}
