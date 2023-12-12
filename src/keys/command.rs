use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ropey::Rope;

use crate::frontend::{
    app::{App, Mode},
    window::Window,
};

fn do_write(window: &Window, path: &str) -> io::Result<()> {
    window.text.write_to(BufWriter::new(File::create(path)?))?;
    Ok(())
}

pub fn process_keys_dialog(event: KeyEvent, app: &mut App) -> bool {
    if let KeyEventKind::Press = event.kind {
        #[allow(clippy::single_match)]
        match event.code {
            KeyCode::Esc => app.current_mode = Mode::Normal,
            KeyCode::Enter => {
                let old_mode = std::mem::replace(&mut app.current_mode, Mode::Normal);
                if let Mode::Command { buffer, .. } = old_mode {
                    if buffer.is_empty() {
                        app.log.log("Empty buffer, aborting");
                        return false;
                    }

                    let args = buffer.split_whitespace().collect::<Vec<&str>>();
                    if args.is_empty() {
                        app.log.log("Empty buffer, aborting");
                        return false;
                    }

                    match args.as_slice() {
                        ["q!" | "quit!"] => return true,
                        ["q" | "quit"] => {
                            if app.has_modified_windows() {
                                app.log.log(
                                    "There are unsaved changes! Use q! or quit! to force quit.",
                                );
                            } else {
                                return true;
                            }
                        }
                        ["c" | "close"] => {
                            if app.selected_window().is_some() {
                                let m = app.selected_window().unwrap().modified;
                                if m {
                                    app.log.log(format!("There are unsaved changes!"));
                                } else {
                                    let closed = app.close_selected();
                                    app.log.log(format!("Closed {}", closed.resolve_title()));
                                }
                            } else {
                                app.log.log("No window Selected");
                            }
                        }
                        ["a" | "attach", param] => {
                            if let Some(sw) = app.selected_window_mut() {
                                sw.attached_file_path = Some(param.to_string());
                                app.log
                                    .log(format!("Attached the current window to {param}"));
                            } else {
                                app.log.log("No window selected");
                            }
                        }
                        ["w" | "write"] => {
                            let to_log: String;
                            if let Some(sw) = app.selected_window_mut() {
                                if let Some(path) = &sw.attached_file_path {
                                    match do_write(sw, path.as_str()) {
                                        Ok(_) => {
                                            sw.modified = false;
                                            to_log = format!(
                                                "Successfully wrote {} bytes to {}",
                                                sw.text.len_bytes(),
                                                path
                                            );
                                        }
                                        Err(e) => {
                                            to_log = format!(
                                                "Error: Could not write {} to {}: {:?}",
                                                sw.resolve_title(),
                                                path,
                                                e
                                            );
                                        }
                                    }
                                } else {
                                    to_log = "This window is not attached".to_string();
                                }
                            } else {
                                to_log = "Error: No open window".to_string();
                            }

                            app.log.log(to_log);
                        }
                        ["o" | "open", path] => match File::open(path) {
                            Ok(f) => match Rope::from_reader(BufReader::new(f)) {
                                Ok(rope) => {
                                    let window = Window {
                                        text: rope,
                                        attached_file_path: Some(path.to_string()),
                                        cursor_char_index: 0,
                                        ident: None,
                                        scroll_x: 0,
                                        scroll_y: 0,
                                        modified: false,
                                    };
                                    app.edit_windows.push(window);
                                    app.selected_window = app.edit_windows.len() - 1;
                                    app.log.log(format!("Successfully opened {path}"));
                                }
                                Err(e) => app.log.log(format!("Could not open {path}: {:?}", e)),
                            },
                            Err(e) => app.log.log(format!("Could not open {path}: {:?}", e)),
                        },
                        ["settitle", new_title] => {
                            if let Some(sw) = app.selected_window_mut() {
                                sw.ident = Some(new_title.to_string());
                                app.log
                                    .log(format!("Successfully set title to {}", new_title));
                            } else {
                                app.log.log("No window selected");
                            }
                        }
                        _ => app
                            .log
                            .log(format!("Could not find interpretation for {buffer}")),
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
