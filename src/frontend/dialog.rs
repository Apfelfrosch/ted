use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};

use super::app::App;

pub enum Dialog {
    LogDisplay { slice_start: usize, selected: usize },
    Help,
    Windows,
}

impl Dialog {
    pub fn render(&self, app: &App, terminal: &mut Frame<'_>, area: Rect) {
        Clear.render(area, terminal.buffer_mut());
        match self {
            Dialog::Windows => {
                let block = Block::default()
                    .title("Windows")
                    .title_style(Style::new().fg(ratatui::style::Color::Yellow))
                    .borders(Borders::all());
                let mut lines = app
                    .edit_windows
                    .iter()
                    .map(|w| w.ident.clone())
                    .collect::<Vec<String>>();
                lines.sort();
                terminal.render_widget(
                    Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<Line>>())
                        .block(block),
                    area,
                );
            }
            Dialog::LogDisplay {
                slice_start,
                selected,
            } => {
                let lines: Vec<Line> = app
                    .log
                    .take_lines()
                    .skip(*slice_start)
                    .take(area.height as usize - 2)
                    .map(Line::from)
                    .collect();
                let block = Block::default()
                    .title(format!(
                        "Logged messages (Current: {}..{} Total: {})",
                        *slice_start,
                        *slice_start + lines.len(),
                        app.log.len()
                    ))
                    .title_style(Style::new().fg(ratatui::style::Color::Yellow))
                    .borders(Borders::all());
                terminal.render_widget(Paragraph::new(lines).block(block), area);
            }
            Dialog::Help => {
                let block = Block::default()
                    .title("Help")
                    .title_style(Style::new().fg(ratatui::style::Color::Yellow))
                    .borders(Borders::all());
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
