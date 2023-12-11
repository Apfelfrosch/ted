use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind},
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

use self::{
    app::{App, Mode},
    dialog::Dialog,
    window::Window,
};
use crate::log::Log;
mod app;
mod dialog;
mod keys;
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
        log: Log::new(),
        current_mode: Mode::Normal,
    };
    let mut fps = 0;

    loop {
        let len_instances = app.edit_windows.len();
        let now = Instant::now();
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(100),
                    Constraint::Min(1),
                    Constraint::Min(1),
                ])
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

            if let Mode::Dialog {
                which_one: current_dialog,
            } = &app.current_mode
            {
                current_dialog.render(&app, frame, centered_rect(frame.size(), 50, 50));
            }

            //frame.render_widget(Paragraph::new(lines), layout[1]);

            if let Mode::Command { buffer, char_idx } = &app.current_mode {
                frame.set_cursor((char_idx + 1) as u16, layout[1].y);
                frame.render_widget(Paragraph::new(Line::from(format!(":{buffer}"))), layout[1]);
            }

            frame.render_widget(
                Paragraph::new(Line::from(app.current_mode.display_name())),
                layout[2],
            );
        })?;
        let frontend_instance = &mut app.edit_windows[app.selected_window];
        let mut elapsed = now.elapsed().as_micros();
        if elapsed == 0 {
            elapsed = 1;
        }
        fps = 1000000 / elapsed;

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                if keys::process_keys(key, &mut app) {
                    break;
                }
            }
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
