use std::{fs::File, io::BufWriter};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::frontend::app::{App, Mode};

pub fn process_keys_dialog(event: KeyEvent, app: &mut App) -> bool {
    if let KeyEventKind::Press = event.kind {
        match event.code {
            KeyCode::Esc => app.current_mode = Mode::Normal,
            KeyCode::Enter => {
                let old_mode = std::mem::replace(&mut app.current_mode, Mode::Normal);
                if let Mode::Command { buffer, .. } = old_mode {
                    app.log.log(format!("Executing {buffer}..."));

                    if buffer.is_empty() {
                        return false;
                    }

                    if buffer.starts_with("q") {
                        return true;
                    }

                    if buffer.starts_with("w") {
                        if let Some((_, arg)) = buffer.split_once(" ") {
                            app.log.log(format!("Writing to {arg}"));
                            app.selected_window()
                                .unwrap()
                                .e
                                .text
                                .write_to(BufWriter::new(File::create(arg).unwrap()))
                                .unwrap();
                        }
                    }
                } else {
                    app.log.log("Error: Not in command mode");
                }
            }
            KeyCode::Left => {
                if let Mode::Command { char_idx, .. } = &mut app.current_mode {
                    if *char_idx > 0 {
                        *char_idx -= 1;
                    }
                } else {
                    app.log.log("Error: Not in command mode");
                }
            }
            KeyCode::Right => {
                if let Mode::Command { buffer, char_idx } = &mut app.current_mode {
                    if *char_idx + 1 < buffer.len() {
                        *char_idx += 1;
                    }
                } else {
                    app.log.log("Error: Not in command mode");
                }
            }
            KeyCode::Backspace => {
                if let Mode::Command { buffer, char_idx } = &mut app.current_mode {
                    if *char_idx > 0 {
                        buffer.remove(*char_idx - 1);
                        *char_idx -= 1;
                    }
                } else {
                    app.log.log("Error: Not in command mode");
                }
            }
            KeyCode::Char(c) => {
                if let Mode::Command { buffer, char_idx } = &mut app.current_mode {
                    buffer.insert(*char_idx, c);
                    *char_idx += 1;
                } else {
                    app.log.log("Error: Not in command mode");
                }
            }
            _ => {}
        }
    }
    false
}
