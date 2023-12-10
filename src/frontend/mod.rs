use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
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

use self::window::Window;
use crate::log::Log;
mod window;

fn initialize_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(stderr(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

pub fn run() -> Result<(), Box<dyn Error>> {
    initialize_panic_hook();

    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;
    terminal.clear()?;

    let mut instances = Vec::new();
    instances.push(Window::default());

    #[allow(unused_mut)]
    let mut selected_instance = 0;

    let mut log = Log::new();
    let mut fps = 0;

    loop {
        let len_instances = instances.len();
        let now = Instant::now();
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                    Constraint::Min(1),
                ])
                .split(frame.size());
            frame.set_cursor(
                instances[selected_instance].cursor_visual_pos_x as u16,
                instances[selected_instance].cursor_visual_pos_y as u16,
            );

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
                instances[idx].render(frame, instances_layout[idx]);
            }

            let mut lines: Vec<Line> = log
                .take_lines(layout[1].height as usize)
                .map(Line::from)
                .collect();
            lines.insert(0, Line::from(format!("FPS: {fps}")));

            frame.render_widget(Paragraph::new(lines), layout[1]);
            frame.render_widget(Paragraph::new(Line::from("STATUS")), layout[2]);
        })?;
        let frontend_instance = &mut instances[selected_instance];
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
                                instances.pop();
                            }
                            'n' => {
                                instances.push(Window::default());
                            }
                            'l' => {
                                if frontend_instance.e.text.len_chars() == 0
                                    || frontend_instance.cursor_char_index
                                        == frontend_instance.e.text.len_chars() - 1
                                {
                                    log.log("At the end (cmd: l)");
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
                                                    log.log("\\r\\n Found (cmd: l)");
                                                    found_line_end = true;
                                                }
                                            }
                                        } else if next_char == '\n' {
                                            log.log("\\n Found (cmd: l)");
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
                                        log.log("Could not get next char (cmd: l)");
                                    }
                                } else {
                                    log.log("Could not get char under cursor (cmd: l)");
                                }
                            }
                            _any => {}
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
