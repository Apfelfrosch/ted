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
    collections::HashMap,
    env,
    error::Error,
    fs::File,
    io::{stderr, BufReader},
    sync::mpsc::{self, Sender, TryRecvError},
    thread,
    time::Duration,
};
use tree_sitter_highlight::{HighlightEvent, Highlighter};

use self::{
    app::{App, Mode},
    window::{HighlightData, HighlightJob, HighlightJobResult},
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

    let (send_hl_job, recv_hl_job) = mpsc::channel::<HighlightJob>();
    let (send_hl_job_result, recv_hl_job_result) = mpsc::channel::<HighlightJobResult>();

    thread::spawn(move || {
        let mut window_uuid_to_cancel_sender = HashMap::<usize, Sender<()>>::new();
        let send_hl_job_result = send_hl_job_result;
        loop {
            let highlight_job = recv_hl_job.recv();
            if highlight_job.is_err() {
                return;
            }
            let highlight_job = highlight_job.unwrap();
            if let Some(cancel_sender) =
                window_uuid_to_cancel_sender.remove(&highlight_job.window_uuid)
            {
                let _ = cancel_sender.send(());
            }
            let send_hl_job_result = send_hl_job_result.clone();
            let (cancel_sender, cancel_recv) = mpsc::channel::<()>();
            window_uuid_to_cancel_sender.insert(highlight_job.window_uuid, cancel_sender);
            thread::spawn(move || {
                let mut highlighter = Highlighter::new();
                let text = highlight_job.text.to_string();
                let highlight_config = highlight_job
                    .language
                    .build_highlighter_config()
                    .expect("Could not build job hl config");
                let mut v: Vec<(usize, std::ops::Range<usize>, &str)> = Vec::new();
                let highlights = highlighter
                    .highlight(&highlight_config, text.as_bytes(), None, |_| None)
                    .unwrap();

                let mut last_token_type = None;
                for event in highlights {
                    match cancel_recv.try_recv() {
                        Ok(_) => return,
                        Err(TryRecvError::Disconnected) => return,
                        _ => match event.unwrap() {
                            HighlightEvent::Source { start, end } => {
                                if let Some(last_token_type) = last_token_type {
                                    let elem = (start, (start..end), last_token_type);
                                    v.push(elem);
                                }
                            }
                            HighlightEvent::HighlightStart(tree_sitter_highlight::Highlight(
                                token_index,
                            )) => {
                                last_token_type = Some(
                                    crate::frontend::language::HIGHLIGHTED_TOKENS[token_index],
                                );
                            }
                            HighlightEvent::HighlightEnd => {
                                last_token_type = None;
                            }
                        },
                    }
                }
                v.sort_by(|(key1, ..), (key2, ..)| key1.cmp(key2));
                send_hl_job_result
                    .send(HighlightJobResult {
                        window_uuid: highlight_job.window_uuid,
                        highlights: v,
                    })
                    .expect("Could not send job result");
            });
        }
    });

    let mut app = App {
        uuid_counter: 0,
        edit_windows: Vec::new(),
        selected_window: 0,
        log: Log::new(),
        current_mode: Mode::Normal,
        highlight_job_queue: send_hl_job,
    };

    let mut args = env::args();
    if let Some(path) = args.nth(1) {
        let mut x = |rope: Rope| {
            let window_index = app.create_empty_window();
            let window = &mut app.edit_windows[window_index];
            window.text = rope;
            window.attached_file_path = Some(path.clone());
            if let Some(lang) = window.try_detect_langauge() {
                app.log
                    .log(format!("[STARTUP] Detected {}", lang.display_name()));
            } else {
                app.log.log("[STARTUP] Couldn't detect language");
            }
            app.queue_selected_window_highlight_refresh();
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
        let hl_job_result = recv_hl_job_result.recv_timeout(Duration::from_micros(1));

        if let Ok(hl_job_result) = hl_job_result {
            for w in app.edit_windows.iter_mut() {
                if hl_job_result.window_uuid == w.uuid {
                    w.highlight_data = Some(HighlightData::new(hl_job_result.highlights));
                    break;
                }
            }
        }

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
