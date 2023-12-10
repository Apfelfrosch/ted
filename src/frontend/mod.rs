use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::Rect,
    prelude::{Constraint, CrosstermBackend, Direction, Layout},
    text::Line,
    widgets::Paragraph,
    Terminal,
};
use std::{
    error::Error,
    io::stderr,
    time::{Duration, Instant},
};

use self::{app::App, window::Window};
use crate::log::Log;
mod app;
mod dialog;
mod window;

fn initialize_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(stderr(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn run() -> Result<(), Box<dyn Error>> {
    initialize_panic_hook();

    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;
    terminal.clear()?;

    let mut app = App {
        edit_windows: vec![Window::default()],
        selected_window: 0,
        current_dialog: Some(dialog::Dialog::LogDisplay {
            slice_start: 0,
            selected: 0,
        }),
        log: Log::new(),
    };
    let mut fps = 0;

    loop {
        let len_instances = app.edit_windows.len();
        let now = Instant::now();
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100), Constraint::Min(1)])
                .split(frame.size());

            let mut contraints_instances = Vec::new();
            let size_per_instance = 100 / len_instances;
            for _ in 0..len_instances {
                contraints_instances.push(Constraint::Percentage(size_per_instance as u16));
            }
            let instances_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(contraints_instances)
                .split(layout[0]);
            for idx in 0..len_instances {
                app.edit_windows[idx].render(frame, instances_layout[idx]);
            }
            app.edit_windows[app.selected_window]
                .render_cursor(frame, instances_layout[app.selected_window]);

            if let Some(dialog) = &app.current_dialog {
                dialog.render(&app, frame, centered_rect(frame.size(), 50, 50));
            }

            //frame.render_widget(Paragraph::new(lines), layout[1]);
            frame.render_widget(Paragraph::new(Line::from("STATUS")), layout[1]);
        })?;
        let frontend_instance = &mut app.edit_windows[app.selected_window];
        let mut elapsed = now.elapsed().as_micros();
        if elapsed == 0 {
            elapsed = 1;
        }
        fps = 1000000 / elapsed;

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let KeyCode::Char(c) = key.code {
                        match c {
                            'q' => break,
                            'd' => {
                                app.edit_windows.pop();
                            }
                            'n' => {
                                let mut w = Window::default();
                                w.ident = format!("Window #{}", len_instances + 1);
                                app.edit_windows.push(w);
                            }
                            'w' => app.selected_window += 1,
                            'l' => {
                                if frontend_instance.e.text.len_chars() == 0
                                    || frontend_instance.cursor_char_index
                                        == frontend_instance.e.text.len_chars() - 1
                                {
                                    app.log.log("At the end (cmd: l)");
                                    continue;
                                }

                                let char_under_cursor = frontend_instance
                                    .e
                                    .text
                                    .get_char(frontend_instance.cursor_char_index);
                                if let Some(char_under_cursor) = char_under_cursor {
                                    let next_char = frontend_instance
                                        .e
                                        .text
                                        .get_char(frontend_instance.cursor_char_index + 1);
                                    if let Some(next_char) = next_char {
                                        let mut found_line_end = false;
                                        if next_char == '\r' {
                                            if let Some(next_next_char) = frontend_instance
                                                .e
                                                .text
                                                .get_char(frontend_instance.cursor_char_index + 2)
                                            {
                                                if next_next_char == '\n' {
                                                    app.log.log("\\r\\n Found (cmd: l)");
                                                    found_line_end = true;
                                                }
                                            }
                                        } else if next_char == '\n' {
                                            app.log.log("\\n Found (cmd: l)");
                                            found_line_end = true;
                                        }

                                        if !found_line_end {
                                            frontend_instance.cursor_char_index += 1;
                                            frontend_instance.cursor_visual_pos_x +=
                                                unicode_width::UnicodeWidthChar::width(
                                                    char_under_cursor,
                                                )
                                                .unwrap_or(1);
                                        }
                                    } else {
                                        app.log.log("Could not get next char (cmd: l)");
                                    }
                                } else {
                                    app.log.log("Could not get char under cursor (cmd: l)");
                                }
                            }
                            any => {
                                app.log.log(any.to_string());
                                frontend_instance
                                    .e
                                    .text
                                    .insert_char(frontend_instance.cursor_char_index, any);
                                frontend_instance.cursor_visual_pos_x +=
                                    unicode_width::UnicodeWidthChar::width(any).unwrap_or(1);
                                frontend_instance.cursor_char_index += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
