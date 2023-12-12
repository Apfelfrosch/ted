use std::{
    fmt::Display,
    fs::File,
    io::{self, BufReader, BufWriter},
    path::Path,
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
                    app.log.log(format!("Executing {buffer}..."));

                    if buffer.is_empty() {
                        return false;
                    }

                    let args = buffer.split_whitespace().collect::<Vec<&str>>();
                    if args.is_empty() {
                        return false;
                    }

                    match args.as_slice() {
                        ["q" | "quit"] => return true,
                        ["a", param] | ["attach", param] => {
                            if let Some(sw) = app.selected_window_mut() {
                                sw.attached_file_path = Some(param.to_string());
                                app.log
                                    .log(format!("Attached the current window to {param}"));
                            } else {
                                app.log.log("No window selected");
                            }
                        }
                        ["w"] | ["write"] => {
                            if let Some(sw) = app.selected_window() {
                                if let Some(path) = &sw.attached_file_path {
                                    match do_write(sw, path.as_str()) {
                                        Ok(_) => app.log.log(format!(
                                            "Successfully wrote {} bytes to {}",
                                            sw.text.len_bytes(),
                                            path
                                        )),
                                        Err(e) => {
                                            app.log.log(format!(
                                                "Error: Could not write {} to {}: {:?}",
                                                sw.resolve_title(),
                                                path,
                                                e
                                            ));
                                        }
                                    }
                                }
                            } else {
                                app.log.log("Error: No open window");
                            }
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
                        _ => {}
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
