use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::frontend::{
    app::{App, Mode},
    dialog::Dialog,
};

pub fn process_keys_normal(event: KeyEvent, app: &mut App) -> bool {
    if let KeyEventKind::Press = event.kind {
        match event.code {
            KeyCode::Char(c) => match c {
                ':' => {
                    app.current_mode = Mode::Command {
                        buffer: String::new(),
                        char_idx: 0,
                    }
                }
                'a' => app.selected_window = if app.selected_window == 0 { 1 } else { 0 },
                'i' => app.current_mode = Mode::Insert,
                'l' => {
                    if let Some(sw) = app.selected_window_mut() {
                        if sw.cursor_char_index + 1 >= sw.text.len_chars() {
                            return false;
                        }
                        sw.cursor_char_index += 1;
                    }
                }
                'h' => {
                    if let Some(sw) = app.selected_window_mut() {
                        if sw.cursor_char_index == 0 {
                            return false;
                        }
                        sw.cursor_char_index -= 1;
                    }
                }
                'j' => {
                    if let Some(sw) = app.selected_window_mut() {
                        let text = &mut sw.text;
                        let current_line_index = text.char_to_line(sw.cursor_char_index);
                        if current_line_index + 1 >= text.len_lines() {
                            return false;
                        }
                        let start_of_next_line = text.line_to_char(current_line_index + 1);
                        sw.cursor_char_index = start_of_next_line;
                    }
                }
                'k' => {
                    if let Some(sw) = app.selected_window_mut() {
                        let text = &mut sw.text;
                        let current_line_index = text.char_to_line(sw.cursor_char_index);
                        if current_line_index == 0 {
                            return false;
                        }
                        let start_of_prev_line = text.line_to_char(current_line_index - 1);
                        sw.cursor_char_index = start_of_prev_line;
                    }
                }
                'L' => {
                    app.current_mode = Mode::Dialog {
                        which_one: Dialog::LogDisplay {
                            slice_start: 0,
                            selected: 0,
                        },
                    }
                }
                'H' => {
                    app.current_mode = Mode::Dialog {
                        which_one: Dialog::Help,
                    }
                }
                'W' => {
                    app.current_mode = Mode::Dialog {
                        which_one: Dialog::Windows,
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    false
}
