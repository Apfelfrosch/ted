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
}

impl Dialog {
    pub fn render(&self, app: &App, terminal: &mut Frame<'_>, area: Rect) {
        Clear.render(area, terminal.buffer_mut());
        match self {
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
        }
    }
}
