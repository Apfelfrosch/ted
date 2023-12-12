use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use unicode_width::UnicodeWidthChar;

use crate::frontend::{
    app::{App, Mode},
    dialog::Dialog,
};

mod command;
mod dialog;
mod insert;
mod normal;

pub fn process_keys(event: KeyEvent, app: &mut App) -> bool {
    match &app.current_mode {
        Mode::Normal => normal::process_keys_normal(event, app),
        Mode::Insert => insert::process_keys_insert(event, app),
        Mode::Dialog { .. } => dialog::process_keys_dialog(event, app),
        Mode::Command { .. } => command::process_keys_dialog(event, app),
    }
}
