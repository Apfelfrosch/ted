use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::frontend::app::{App, Mode};

pub fn process_keys_dialog(event: KeyEvent, app: &mut App) -> bool {
    if let KeyEventKind::Press = event.kind {
        match event.code {
            KeyCode::Esc => app.current_mode = Mode::Normal,
            _ => {}
        }
    }
    false
}
