use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

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
                        sw.cursor_visual_pos_x = 0;
                        sw.cursor_visual_pos_y += 1;
                    }
                }
                KeyCode::Backspace if app.current_mode.is_insert() => {
                    if let Some(sw) = app.selected_window_mut() {
                        if sw.cursor_char_index != 0 {
                            let current_char = sw.e.text.char(sw.cursor_char_index);
                            if current_char == '\n' {
                                return false;
                            }
                            sw.e.text
                                .remove(sw.cursor_char_index..=sw.cursor_char_index);
                            sw.cursor_char_index -= 1;

                            let current_char = sw.e.text.char(sw.cursor_char_index);
                            sw.cursor_visual_pos_x -=
                                unicode_width::UnicodeWidthChar::width(current_char).unwrap_or(1);
                        }
                    }
                }
                KeyCode::Esc if app.current_mode.is_insert() => app.current_mode = Mode::Normal,
                KeyCode::Char(c) if app.current_mode.is_normal() => match c {
                    'i' => app.current_mode = Mode::Insert,
                    'q' => return true,
                    'l' => {}
                    'L' if app.current_dialog.is_none() => {
                        app.current_dialog = Some(Dialog::LogDisplay {
                            slice_start: 0,
                            selected: 0,
                        })
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        _ => {}
    }
    false
}
