use ropey::Rope;

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
    pub fn has_modified_windows(&self) -> bool {
        self.edit_windows.iter().any(|w| w.modified)
    }

    pub fn create_empty_window(&mut self) -> usize {
        let window = Window {
            attached_file_path: None,
            cursor_char_index: 0,
            ident: None,
            modified: false,
            scroll_x: 0,
            scroll_y: 0,
            text: Rope::new(),
            language: None,
            highlight_data: None,
        };
        self.edit_windows.push(window);
        self.edit_windows.len() - 1
    }

    pub fn close_selected(&mut self) -> Window {
        let w = self.edit_windows.remove(self.selected_window);
        if self.edit_windows.is_empty() {
            self.selected_window = 0
        } else if self.selected_window >= self.edit_windows.len() {
            self.selected_window = self.edit_windows.len() - 1;
        } else if self.selected_window > 0 {
            self.selected_window -= 1;
        }
        w
    }

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

    pub fn previous_window(&mut self) {
        if self.selected_window > 0 {
            self.selected_window -= 1;
        } else if !self.edit_windows.is_empty() {
            self.selected_window = self.edit_windows.len() - 1;
        }
    }

    pub fn next_window(&mut self) {
        self.selected_window += 1;
        if self.selected_window >= self.edit_windows.len() {
            self.selected_window = 0;
        }
    }
}
