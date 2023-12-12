use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::frontend::{
    app::{App, Mode},
    dialog::Dialog,
};

pub fn process_keys_dialog(event: KeyEvent, app: &mut App) -> bool {
    if let KeyEventKind::Press = event.kind {
        if let Mode::Dialog { which_one } = &app.current_mode {
            #[allow(clippy::single_match)]
            match event.code {
                KeyCode::Esc | KeyCode::Enter => app.current_mode = Mode::Normal,
                KeyCode::Left
                | KeyCode::Char('h')
                | KeyCode::Char('H')
                | KeyCode::Char('k')
                | KeyCode::Up => {
                    if let Dialog::Windows { .. } = which_one {
                        app.previous_window();
                    }
                }
                KeyCode::Right
                | KeyCode::Char('l')
                | KeyCode::Char('L')
                | KeyCode::Char('j')
                | KeyCode::Down => {
                    if let Dialog::Windows { .. } = which_one {
                        app.next_window();
                    }
                }
                _ => {}
            }
        } else {
            app.log.log("Error: Not in dialog mode");
        }
    }
    false
}
