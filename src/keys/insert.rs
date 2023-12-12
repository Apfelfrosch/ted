use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::frontend::app::{App, Mode};

pub fn process_keys_insert(event: KeyEvent, app: &mut App) -> bool {
    if let KeyEventKind::Press = event.kind {
        match event.code {
            KeyCode::Enter => {
                if let Some(sw) = app.selected_window_mut() {
                    sw.e.text.insert_char(sw.cursor_char_index, '\n');
                    sw.cursor_char_index += 1;
                }
            }
            KeyCode::Backspace => {
                if let Some(sw) = app.selected_window_mut() {
                    if sw.cursor_char_index > 0 {
                        sw.e.text
                            .remove((sw.cursor_char_index - 1)..sw.cursor_char_index);
                        sw.cursor_char_index -= 1;
                    }
                }
            }
            KeyCode::Esc => app.current_mode = Mode::Normal,
            KeyCode::Char(c) => match c {
                any => {
                    if let Some(sw) = app.selected_window_mut() {
                        sw.e.text.insert_char(sw.cursor_char_index, any);
                        sw.cursor_char_index += 1;
                    }
                }
            },
            _ => {}
        }
    }
    false
}
