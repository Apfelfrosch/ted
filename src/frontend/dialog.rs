use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};

use super::{app::App, COMMAND_MODE_BACKGROUND};

pub enum Dialog {
    Logs,
    Help,
    Windows,
}

impl Dialog {
    fn create_block() -> Block<'static> {
        Block::default()
            .title_style(Style::new().fg(ratatui::style::Color::Yellow))
            .borders(Borders::all())
    }

    pub fn render(&self, app: &App, terminal: &mut Frame<'_>, area: Rect) {
        Clear.render(area, terminal.buffer_mut());
        match self {
            Dialog::Windows => {
                const SCROLL_BORDERS: usize = 3;

                let mut to_skip = if app.selected_window < SCROLL_BORDERS {
                    0
                } else {
                    app.selected_window - SCROLL_BORDERS
                };

                let last_window_seen =
                    (to_skip + area.height as usize - 2).min(app.edit_windows.len());
                if last_window_seen > area.height as usize + 2
                    && last_window_seen == app.edit_windows.len()
                {
                    to_skip = last_window_seen - area.height as usize - 2 + SCROLL_BORDERS + 1;
                }
                let block = Dialog::create_block().title(format!("Windows {last_window_seen}"));

                let lines = app
                    .edit_windows
                    .iter()
                    .skip(to_skip)
                    .take(area.height as usize)
                    .enumerate()
                    .map(|(idx, window)| {
                        let is_selcted = app.selected_window == idx + to_skip;

                        let mut spans = vec![
                            Span::from(window.resolve_title()),
                            Span::from(" "),
                            Span::from("[Attached: "),
                            if let Some(p) = &window.attached_file_path {
                                Span::from(p).fg(Color::Green)
                            } else {
                                Span::from("Not Attached").fg(Color::Red)
                            },
                            Span::from(" "),
                            Span::from("Modified: "),
                            if window.modified {
                                Span::from("true").fg(Color::Red)
                            } else {
                                Span::from("false").fg(Color::Green)
                            },
                            Span::from("]"),
                        ];

                        if is_selcted {
                            spans = spans
                                .into_iter()
                                .map(|s| s.bg(COMMAND_MODE_BACKGROUND))
                                .collect();
                        }

                        Line::from(spans)
                    })
                    .collect::<Vec<Line>>();
                terminal.render_widget(Paragraph::new(lines).block(block), area);
            }
            Dialog::Logs => {
                let lines: Vec<Line> = app
                    .log
                    .take_lines()
                    .take(area.height as usize - 2)
                    .map(Line::from)
                    .collect();
                let block = Dialog::create_block().title("Log");
                terminal.render_widget(Paragraph::new(lines).block(block), area);
            }
            Dialog::Help => {
                let block = Dialog::create_block().title("Help");
                terminal.render_widget(
                    Paragraph::new(
                        [
                            "Normal Mode",
                            "q - quit",
                            "L - display log",
                            "H - display this help",
                            "i - go into insert mode",
                            "h - go right",
                            "j - go down",
                            "k - go up",
                            "l - go right",
                            "<Esc> - close dialogs",
                        ]
                        .iter()
                        .map(ToString::to_string)
                        .map(Line::from)
                        .collect::<Vec<Line>>(),
                    )
                    .block(block),
                    area,
                );
            }
        }
    }
}
