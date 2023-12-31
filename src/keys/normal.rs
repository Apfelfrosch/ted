use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::frontend::{
    app::{App, Mode},
    dialog::Dialog,
};

pub fn process_keys_normal(event: KeyEvent, app: &mut App) -> bool {
    if let KeyEventKind::Press = event.kind {
        #[allow(clippy::single_match)]
        match event.code {
            KeyCode::Char(c) => match c {
                ':' => {
                    app.current_mode = Mode::Command {
                        buffer: String::new(),
                        char_idx: 0,
                    }
                }
                'a' => {
                    if let Some(sw) = app.selected_window_mut() {
                        if sw.cursor_char_index < sw.text.len_chars() {
                            sw.cursor_char_index += 1;
                        }
                        app.current_mode = Mode::Insert;
                    }
                }
                'i' => {
                    if app.edit_windows.is_empty() {
                        app.log.log("Error: No open windows");
                    } else {
                        app.current_mode = Mode::Insert;
                    }
                }
                'x' => {
                    if let Some(sw) = app.selected_window_mut() {
                        if !(sw.text.len_chars() == 0
                            || sw.cursor_char_index >= sw.text.len_chars())
                        {
                            sw.text.remove(sw.cursor_char_index..=sw.cursor_char_index);
                            app.queue_selected_window_highlight_refresh();
                        }
                    }
                }
                's' => {
                    if let Some(sw) = app.selected_window_mut() {
                        if !(sw.text.len_chars() == 0
                            || sw.cursor_char_index >= sw.text.len_chars())
                        {
                            sw.text.remove(sw.cursor_char_index..=sw.cursor_char_index);
                            app.current_mode = Mode::Insert;
                            app.queue_selected_window_highlight_refresh();
                        }
                    }
                }
                'O' => {
                    if let Some(sw) = app.selected_window_mut() {
                        let line_index = sw.text.char_to_line(sw.cursor_char_index);
                        let idx = sw.text.line_to_char(line_index);
                        sw.text.insert(idx, "\n");
                        sw.cursor_char_index = sw.text.line_to_char(line_index);

                        app.current_mode = Mode::Insert;
                        app.queue_selected_window_highlight_refresh();
                    }
                }
                'o' => {
                    if let Some(sw) = app.selected_window_mut() {
                        let line_index = sw.text.char_to_line(sw.cursor_char_index);
                        let line_slice = sw.text.line(line_index);
                        let idx = sw.text.line_to_char(line_index) + line_slice.len_chars();
                        sw.text.insert(idx, "\n");
                        sw.cursor_char_index = sw.text.line_to_char(line_index + 1);

                        app.current_mode = Mode::Insert;
                        app.queue_selected_window_highlight_refresh();
                    }
                }
                'G' => {
                    if let Some(sw) = app.selected_window_mut() {
                        if sw.text.len_chars() != 0 {
                            sw.cursor_char_index = sw.text.len_chars() - 1;
                        }
                    }
                }
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

                        let line_offset =
                            sw.cursor_char_index - text.line_to_char(current_line_index);
                        let next_line_slice = text.line(current_line_index + 1);
                        let new_offset = line_offset.min(next_line_slice.len_chars());

                        sw.cursor_char_index = start_of_next_line + new_offset;
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
                'L' => app.next_window(),
                'H' => app.previous_window(),
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
