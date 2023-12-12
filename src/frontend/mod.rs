use crossterm::{
    event::{self},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::Rect,
    prelude::{Constraint, CrosstermBackend, Direction, Layout},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use ropey::Rope;
use std::{
    env,
    error::Error,
    fs::File,
    io::{stderr, BufReader},
    time::Duration,
};

use self::{
    app::{App, Mode},
    window::Window,
};
use crate::log::Log;

pub mod app;
pub mod dialog;
pub mod language;
pub mod window;

const COMMAND_MODE_BACKGROUND: Color = Color::Rgb(77, 77, 77);

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
        edit_windows: Vec::new(),
        selected_window: 0,
        log: Log::new(),
        current_mode: Mode::Normal,
    };

    let mut args = env::args();
    if let Some(path) = args.nth(1) {
        let mut x = |rope: Rope| {
            let mut window = Window {
                text: rope,
                attached_file_path: Some(path.clone()),
                cursor_char_index: 0,
                ident: None,
                scroll_x: 0,
                scroll_y: 0,
                modified: false,
                highlight_data: None,
                language: None,
            };
            if let Some(lang) = window.try_detect_langauge() {
                app.log
                    .log(format!("[STARTUP] Detected {}", lang.display_name()));
            } else {
                app.log.log(format!("[STARTUP] Couldn't detect language"));
            }
            app.edit_windows.push(window);
        };

        match File::open(&path) {
            Ok(f) => match Rope::from_reader(BufReader::new(f)) {
                Ok(rope) => {
                    x(rope);
                    app.selected_window = app.edit_windows.len() - 1;
                    app.log.log(format!("[STARTUP] Successfully opened {path}"));
                }
                Err(e) => {
                    x(Rope::new());
                    app.log.log(format!(
                        "[STARTUP] Could not open {path} due to {:?} -> created empty window",
                        e
                    ));
                }
            },
            Err(e) => {
                x(Rope::new());
                app.log.log(format!(
                    "[STARTUP] Could not open {path} due to {:?} -> created empty window",
                    e
                ));
            }
        }
    }

    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(100),
                    Constraint::Min(1),
                    Constraint::Min(1),
                ])
                .split(frame.size());

            if app.edit_windows.is_empty() {
                let edit_area = layout[0];
                let center_of_edit_area = centered_rect(edit_area, 50, 50);
                frame.render_widget(
                    Paragraph::new(vec![
                        Line::from("There are no open windows."),
                        Line::from("Open a file with :o"),
                        Line::from("Create an empty window with :n"),
                    ])
                    .alignment(ratatui::layout::Alignment::Center),
                    center_of_edit_area,
                );
            } else {
                if app.selected_window >= app.edit_windows.len() {
                    app.selected_window = app.edit_windows.len() - 1;
                }

                if let Some(sw) = app.selected_window_mut() {
                    sw.render(frame, layout[0], true);
                    sw.render_cursor(frame, layout[0]);
                }
            }

            let mut is_command_mode = false;
            match &app.current_mode {
                Mode::Dialog {
                    which_one: current_dialog,
                } => {
                    current_dialog.render(&app, frame, centered_rect(frame.size(), 50, 50));
                }
                Mode::Command { buffer, char_idx } => {
                    is_command_mode = true;
                    frame.set_cursor((char_idx + 1) as u16, layout[1].y);
                    frame.render_widget(
                        Paragraph::new(Line::from(format!(":{buffer}")))
                            .bg(COMMAND_MODE_BACKGROUND),
                        layout[1],
                    );
                }
                _ => {}
            }

            if !is_command_mode {
                if let Some(first_log_line) = app.log.take_lines().next() {
                    frame.render_widget(Paragraph::new(Line::from(first_log_line)), layout[1]);
                }
            }

            let status_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(layout[2]);

            frame.render_widget(
                Paragraph::new(Line::from(Span::from(app.current_mode.display_name()))),
                status_layout[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::from(app.current_mode.display_name())))
                    .alignment(ratatui::layout::Alignment::Right),
                status_layout[1],
            );
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                if crate::keys::process_keys(key, &mut app) {
                    break;
                }
            }
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
