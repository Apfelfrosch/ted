use crate::log::Log;

use super::{dialog::Dialog, window::Window};

pub struct App {
    pub edit_windows: Vec<Window>,
    pub selected_window: usize,
    pub current_dialog: Option<Dialog>,
    pub log: Log,
}
