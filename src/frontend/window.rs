use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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
}

fn visual_length_of_number(i: usize) -> u32 {
    if i == 0 {
        1
    } else {
        i.ilog10() + 1
    }
}

impl Window {
    pub fn render(&self, terminal: &mut Frame<'_>, layout_rect: Rect) {
        let max_lines = visual_length_of_number(self.e.text.len_lines());

        let v = self
            .e
            .text
            .lines_at(self.scroll_y)
            .enumerate()
            .take(layout_rect.height as usize)
            .fold(Vec::new(), |mut acc, (idx, element)| {
                let mut line_buf = String::with_capacity(max_lines as usize + 1);

                let idx = idx + 1;
                for _ in 0..(max_lines - visual_length_of_number(idx) as u32) {
                    line_buf.push('0');
                }

                line_buf.push_str(&idx.to_string());
                line_buf.push(' ');

                if self.scroll_x >= element.len_chars() {
                    acc.push(Line::from(line_buf));
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
                acc.push(Line::from(format!(
                    "{}{}",
                    line_buf,
                    line_string.replace('\n', "␊")
                )));
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
        let current_line = self.e.text.char_to_line(self.cursor_char_index);
        if current_line < self.scroll_y {
            return;
        }
        let max_lines_seen = self.scroll_y + layout_rect.height as usize - 2;
        if current_line > max_lines_seen {
            return;
        }

        let current_line_start = self.e.text.line_to_char(current_line);
        let cursor_y = current_line - self.scroll_y + 1;
        let mut cursor_x = (1 + visual_length_of_number(self.e.text.len_lines()) + 1) as usize;
        for i in 0..(self.cursor_char_index - current_line_start) {
            cursor_x +=
                unicode_width::UnicodeWidthChar::width(self.e.text.char(current_line_start + i))
                    .unwrap_or(1);
        }
        terminal.set_cursor(cursor_x as u16, cursor_y as u16);
    }
}

impl Default for Window {
    fn default() -> Self {
        Window {
            ident: "Window".to_string(),
            e: Editor {
                text: Rope::from_reader(BufReader::new(File::open("test_text.txt").unwrap()))
                    .unwrap(),
            },
            scroll_x: 0,
            scroll_y: 0,
            cursor_char_index: 0,
        }
    }
}
