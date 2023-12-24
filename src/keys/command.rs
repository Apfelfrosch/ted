use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter},
};

use std::io::Write;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ropey::Rope;
use tempfile::NamedTempFile;

use crate::frontend::{
    app::{App, Mode},
    dialog::Dialog,
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
                        ["log" | "logs"] => {
                            app.current_mode = Mode::Dialog {
                                which_one: Dialog::Logs,
                            }
                        }
                        ["c" | "close"] => {
                            if app.selected_window().is_some() {
                                let m = app.selected_window().unwrap().modified;
                                if m {
                                    app.log.log("There are unsaved changes!".to_string());
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
                                sw.try_detect_langauge();
                                app.log
                                    .log(format!("Attached the current window to {param}"));
                            } else {
                                app.log.log("No window selected");
                            }
                            app.queue_selected_window_highlight_refresh();
                        }
                        ["n" | "new"] => app.selected_window = app.create_empty_window(),
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
                            Ok(f) => {
                                let mut reader = BufReader::new(f);
                                let mut text = String::new();
                                let mut line_buf = String::new();
                                let cancel = loop {
                                    line_buf.clear();
                                    let res = reader.read_line(&mut line_buf);
                                    match res {
                                        Ok(0) => break false,
                                        Ok(_) => {
                                            let replaced = line_buf.replace('\t', "    ");
                                            text.push_str(replaced.as_str());
                                        }
                                        Err(e) => {
                                            app.log.log(format!("Could not open {path}: {:?}", e));
                                            break true;
                                        }
                                    }
                                };
                                if !cancel {
                                    let rope = Rope::from_str(text.as_str());

                                    let window_index = app.create_empty_window();
                                    let window = &mut app.edit_windows[window_index];
                                    window.text = rope;
                                    window.attached_file_path = Some(path.to_string());
                                    if let Some(lang) = window.try_detect_langauge() {
                                        app.log.log(format!("Detected {}", lang.display_name()));
                                    } else {
                                        app.log.log("Couldn't detect language");
                                    }
                                    app.selected_window = app.edit_windows.len() - 1;
                                    app.log.log(format!("Successfully opened {path}"));
                                    app.queue_selected_window_highlight_refresh();
                                }
                            }
                            Err(e) => app.log.log(format!("Could not open {path}: {:?}", e)),
                        },
                        ["format" | "fmt"] => {
                            if let Some(sw) = app.selected_window_mut() {
                                if let Some(lang) = sw.language {
                                    match NamedTempFile::with_prefix("ted_format_") {
                                        Ok(mut f) => match write!(f, "{}", sw.text) {
                                            Ok(_) => {
                                                if let Some(mut cmd) =
                                                    lang.format_command(f.path().to_str().unwrap())
                                                {
                                                    let output =
                                                        cmd.output().expect("Could not format!");
                                                    let mut new_text =
                                                        std::fs::read_to_string(f).unwrap();
                                                    new_text = new_text.replace('\t', "    ");
                                                    sw.text = Rope::from_str(new_text.as_str());
                                                    app.log.log(format!(
                                                        "Successfully formatted! Exit code: {}",
                                                        output.status.code().unwrap_or(0)
                                                    ));
                                                    app.queue_selected_window_highlight_refresh();
                                                }
                                            }
                                            Err(e) => {
                                                app.log.log(format!("Could not format: {:?}", e))
                                            }
                                        },
                                        Err(e) => app.log.log(format!("Could not format: {:?}", e)),
                                    }
                                }
                            }
                        }
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
