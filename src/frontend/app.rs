use crate::log::Log;

use super::{dialog::Dialog, window::Window};

pub enum Mode {
    Normal,
    Insert,
    Dialog { which_one: Dialog },
    Command { buffer: String, char_idx: usize },
}

impl Mode {
    pub fn display_name(&self) -> &'static str {
        match &self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Dialog { .. } => "DIALOG",
            Mode::Command { .. } => "COMMAND",
        }
    }
}

pub struct App {
    pub edit_windows: Vec<Window>,
    pub selected_window: usize,
    pub log: Log,
    pub current_mode: Mode,
}

impl App {
    pub fn selected_window(&self) -> Option<&'_ Window> {
        if self.edit_windows.is_empty() {
            None
        } else {
            Some(&self.edit_windows[self.selected_window])
        }
    }

    pub fn selected_window_mut(&mut self) -> Option<&'_ mut Window> {
        if self.edit_windows.is_empty() {
            None
        } else {
            Some(&mut self.edit_windows[self.selected_window])
        }
    }
}
