use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use unicode_width::UnicodeWidthChar;

use super::{
    app::{App, Mode},
    dialog::Dialog,
};

pub fn process_keys(event: KeyEvent, app: &mut App) -> bool {
    match event.kind {
        KeyEventKind::Press => {
            match event.code {
                KeyCode::Esc if app.current_dialog.is_some() => app.current_dialog = None,

                KeyCode::Enter if app.current_mode.is_insert() => {
                    if let Some(sw) = app.selected_window_mut() {
                        // TODO: SUPPORT CRLN
                        sw.e.text.insert_char(sw.cursor_char_index, '\n');
                        sw.cursor_char_index += 1;
                    }
                }
                KeyCode::Backspace if app.current_mode.is_insert() => {
                    if let Some(sw) = app.selected_window_mut() {
                        if sw.cursor_char_index > 0 {
                            let current_char = sw.e.text.char(sw.cursor_char_index - 1);
                            if current_char == '\n' {
                                return false;
                            }
                            sw.e.text
                                .remove((sw.cursor_char_index - 1)..sw.cursor_char_index);
                            sw.cursor_char_index -= 1;
                        }
                    }
                }
                KeyCode::Esc if app.current_mode.is_insert() => app.current_mode = Mode::Normal,
                KeyCode::Char(c)
                    if app.current_mode.is_insert() && app.current_dialog.is_none() =>
                {
                    match c {
                        any => {
                            if let Some(sw) = app.selected_window_mut() {
                                sw.e.text.insert_char(sw.cursor_char_index, any);
                                sw.cursor_char_index += 1;
                            }
                        }
                    }
                }
                KeyCode::Char(c)
                    if app.current_mode.is_normal() && app.current_dialog.is_none() =>
                {
                    match c {
                        'i' => app.current_mode = Mode::Insert,
                        'q' => return true,
                        'l' => {
                            if let Some(sw) = app.selected_window_mut() {
                                if sw.cursor_char_index + 1 >= sw.e.text.len_chars() {
                                    return false;
                                }

                                let current_char = sw.e.text.char(sw.cursor_char_index);
                                let next_char = sw.e.text.char(sw.cursor_char_index + 1);
                                if next_char == '\n' || next_char == '\r' {
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

                                let prev_char = sw.e.text.char(sw.cursor_char_index - 1);
                                if prev_char == '\n' {
                                    return false;
                                }
                                sw.cursor_char_index -= 1;
                            }
                        }
                        'j' => {
                            if let Some(sw) = app.selected_window_mut() {
                                let text = &mut sw.e.text;
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
                                let text = &mut sw.e.text;
                                let current_line_index = text.char_to_line(sw.cursor_char_index);
                                if current_line_index == 0 {
                                    return false;
                                }
                                let start_of_prev_line = text.line_to_char(current_line_index - 1);
                                sw.cursor_char_index = start_of_prev_line;
                            }
                        }
                        'L' if app.current_dialog.is_none() => {
                            app.current_dialog = Some(Dialog::LogDisplay {
                                slice_start: 0,
                                selected: 0,
                            })
                        }
                        'H' if app.current_dialog.is_none() => {
                            app.current_dialog = Some(Dialog::Help)
                        }
                        'W' if app.current_dialog.is_none() => {
                            app.current_dialog = Some(Dialog::Windows)
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    false
}
