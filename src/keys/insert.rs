use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::frontend::app::{App, Mode};

pub fn process_keys_insert(event: KeyEvent, app: &mut App) -> bool {
    if let KeyEventKind::Press = event.kind {
        #[allow(clippy::single_match)]
        match event.code {
            KeyCode::Enter => {
                if let Some(sw) = app.selected_window_mut() {
                    sw.text.insert_char(sw.cursor_char_index, '\n');
                    sw.cursor_char_index += 1;
                }
            }
            KeyCode::Backspace => {
                if let Some(sw) = app.selected_window_mut() {
                    if sw.cursor_char_index > 0 {
                        sw.text
                            .remove((sw.cursor_char_index - 1)..sw.cursor_char_index);
                        sw.cursor_char_index -= 1;
                    }
                }
            }
            KeyCode::Esc => {
                app.current_mode = Mode::Normal;
                if let Some(sw) = app.selected_window_mut() {
                    sw.refresh_highlighting();
                }
            }
            KeyCode::Char(c) => {
                if let Some(sw) = app.selected_window_mut() {
                    sw.text.insert_char(sw.cursor_char_index, c);
                    sw.modified = true;
                    sw.cursor_char_index += 1;
                }
            }
            _ => {}
        }
    }
    false
}
