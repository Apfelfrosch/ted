use ratatui::{
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ropey::Rope;

use crate::editor::Editor;

pub struct Window {
    pub ident: String,
    pub e: Editor,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub cursor_char_index: usize,
    pub cursor_visual_pos_x: usize,
    pub cursor_visual_pos_y: usize,
}

impl Window {
    pub fn render(&self, terminal: &mut Frame<'_>, layout_rect: Rect) {
        let v = self
            .e
            .text
            .lines_at(self.scroll_y)
            .take(layout_rect.height as usize)
            .fold(Vec::new(), |mut acc, element| {
                if self.scroll_x >= element.len_chars() {
                    acc.push(Line::from(""));
                    return acc;
                }

                #[allow(unused_mut)]
                let mut line_string = element.chars_at(self.scroll_x).collect::<String>();
                /*
                                if line_string.ends_with("\r\n") {
                                    line_string.truncate(line_string.len() - 2);
                                } else if line_string.ends_with("\n") {
                                    line_string.pop();
                                }
                */
                acc.push(Line::from(line_string.replace('\n', "␊")));
                acc
            });
        terminal.render_widget(
            Paragraph::new(v).block(
                Block::default()
                    .title(Line::from(self.ident.as_str()))
                    .borders(Borders::all()),
            ),
            layout_rect,
        );
    }

    pub fn render_cursor(&self, terminal: &mut Frame<'_>, layout_rect: Rect) {
        // +1 because of border
        let cx = layout_rect.x + self.cursor_visual_pos_x as u16 + 1;
        let cy = layout_rect.y + self.cursor_visual_pos_y as u16 + 1;
        terminal.set_cursor(cx, cy);
    }
}

impl Default for Window {
    fn default() -> Self {
        Window {
            ident: "Window".to_string(),
            e: Editor {
                text: Rope::from_str("SOME TEXT\nLine 2\nLine 3"),
            },
            scroll_x: 0,
            scroll_y: 0,
            cursor_char_index: 0,
            cursor_visual_pos_x: 0,
            cursor_visual_pos_y: 0,
        }
    }
}
