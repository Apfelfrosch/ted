use crossterm::{
    event::{self},
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
use std::{error::Error, io::stderr, time::Duration};

use self::app::{App, Mode};
use crate::log::Log;

pub mod app;
pub mod dialog;
pub mod window;

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
        edit_windows: vec![Default::default(), Default::default()],
        selected_window: 0,
        log: Log::new(),
        current_mode: Mode::Normal,
    };

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

            if !app.edit_windows.is_empty() {
                if app.selected_window >= app.edit_windows.len() {
                    app.selected_window = app.edit_windows.len() - 1;
                }

                if let Some(sw) = app.selected_window_mut() {
                    sw.render(frame, layout[0], true);
                    sw.render_cursor(frame, layout[0]);
                }
            }

            match &app.current_mode {
                Mode::Dialog {
                    which_one: current_dialog,
                } => {
                    current_dialog.render(&app, frame, centered_rect(frame.size(), 50, 50));
                }
                Mode::Command { buffer, char_idx } => {
                    frame.set_cursor((char_idx + 1) as u16, layout[1].y);
                    frame
                        .render_widget(Paragraph::new(Line::from(format!(":{buffer}"))), layout[1]);
                }
                _ => {}
            }

            frame.render_widget(
                Paragraph::new(Line::from(app.current_mode.display_name())),
                layout[2],
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
