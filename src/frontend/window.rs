use std::{cmp::Ordering, fs::File, io::BufReader};

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ropey::Rope;
use tree_sitter_highlight::{HighlightEvent, Highlighter};

use super::language::{get_highlight_color, Language};

pub struct Window {
    pub ident: Option<String>,
    pub text: Rope,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub cursor_char_index: usize,
    pub attached_file_path: Option<String>,
    pub modified: bool,
    pub language: Option<Language>,
    pub highlighter: Highlighter,
    pub highlight_data: Option<HighlightData>,
}

pub struct HighlightData {
    highlights: Vec<(usize, std::ops::Range<usize>, &'static str)>,
}

impl HighlightData {
    pub fn find_highlight(&self, byte_index: usize) -> Option<&'static str> {
        match self.highlights.binary_search_by(|(start, range, _)| {
            if range.contains(&byte_index) {
                Ordering::Equal
            } else {
                start.cmp(&byte_index)
            }
        }) {
            Ok(pos) => Some(self.highlights[pos].2),
            Err(_) => None,
        }
    }
}

fn visual_length_of_number(i: usize) -> u32 {
    if i == 0 {
        1
    } else {
        i.ilog10() + 1
    }
}

impl Window {
    pub fn try_detect_langauge(&mut self) -> Option<&Language> {
        if let Some(attached_path) = &self.attached_file_path {
            if let Some(language) = Language::by_file_name(&attached_path) {
                self.language = Some(language);
                self.refresh_highlighting();
            }
        }
        self.language.as_ref()
    }

    pub fn refresh_highlighting(&mut self) {
        if let Some(language) = &self.language {
            if let Some(config) = language.build_highlighter_config() {
                let mut v: Vec<(usize, std::ops::Range<usize>, &str)> = Vec::new();
                let text_as_string = self.text.to_string();
                let highlights = self
                    .highlighter
                    .highlight(&config, text_as_string.as_bytes(), None, |_| None)
                    .unwrap();

                let mut last_token_type = None;
                for event in highlights {
                    match event.unwrap() {
                        HighlightEvent::Source { start, end } => {
                            if let Some(last_token_type) = last_token_type {
                                let elem = (start, (start..end), last_token_type);
                                v.push(elem);
                            }
                        }
                        HighlightEvent::HighlightStart(tree_sitter_highlight::Highlight(
                            token_index,
                        )) => {
                            last_token_type =
                                Some(crate::frontend::language::HIGHLIGHTED_TOKENS[token_index]);
                        }
                        HighlightEvent::HighlightEnd => {
                            last_token_type = None;
                        }
                    }
                }
                v.sort_by(|(key1, ..), (key2, ..)| key1.cmp(key2));
                self.highlight_data = Some(HighlightData { highlights: v });
            }
        }
    }

    pub fn resolve_title(&self) -> &str {
        self.ident
            .as_ref()
            .or(self.attached_file_path.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Untitled")
    }

    pub fn render(&mut self, terminal: &mut Frame<'_>, layout_rect: Rect, _is_selected: bool) {
        if layout_rect.height < 2 {
            return;
        }
        let max_lines = visual_length_of_number(self.text.len_lines());
        let current_line_index = self.text.char_to_line(self.cursor_char_index);
        let max_line_seen = self.scroll_y + layout_rect.height as usize - 3;

        if current_line_index < self.scroll_y {
            let offset = self.scroll_y - current_line_index;
            self.scroll_y -= offset;
        }

        if current_line_index > max_line_seen {
            self.scroll_y += current_line_index - max_line_seen;
        }

        let current_line_start = self.text.line_to_char(current_line_index);
        let line_offset = self.cursor_char_index - current_line_start;

        let l = layout_rect.width as usize - max_lines as usize - 1 - 3;

        if line_offset > l + self.scroll_x {
            self.scroll_x += line_offset - l - self.scroll_x;
        }

        if line_offset < self.scroll_x {
            let offset = self.scroll_x - line_offset;
            self.scroll_x -= offset;
        }

        let v = self
            .text
            .lines_at(self.scroll_y)
            .enumerate()
            .take(layout_rect.height as usize)
            .fold(Vec::new(), |mut acc, (o_idx, element)| {
                let idx = o_idx + self.scroll_y;
                let mut line_buf = String::with_capacity(max_lines as usize + 1);

                let idx = idx + 1;
                for _ in 0..(max_lines - visual_length_of_number(idx)) {
                    line_buf.push('0');
                }

                line_buf.push_str(&idx.to_string());
                line_buf.push(' ');
                let line_span = Span::styled(line_buf, Style::new().fg(Color::Yellow));

                if self.scroll_x >= element.len_chars() {
                    acc.push(Line::from(line_span));
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

                let mut spans = Vec::new();
                spans.push(line_span);

                let start_of_current_line = self.text.line_to_char(o_idx + self.scroll_y);

                for (i, c) in element.chars().enumerate() {
                    let string_to_use = match c {
                        '\n' => '␊'.to_string(),
                        '\t' => "    ".to_string(), // tab as 4 spaces
                        c => c.to_string(),
                    };

                    let byte_index = self
                        .text
                        .try_char_to_byte(start_of_current_line + i)
                        .expect("Byte not found");
                    let mut span = Span::from(string_to_use);
                    if let Some(hd) = &self.highlight_data {
                        if let Some(token) = hd.find_highlight(byte_index) {
                            if let Some(color) = get_highlight_color(token) {
                                span = span.fg(color);
                            }
                        }
                    }
                    spans.push(span);
                }

                acc.push(Line::from(spans));
                acc
            });
        terminal.render_widget(
            Paragraph::new(v).block(
                Block::default()
                    .title(Line::from(format!(
                        "{}{}",
                        self.resolve_title(),
                        if self.modified { "*" } else { "" }
                    )))
                    .borders(Borders::all()),
            ),
            layout_rect,
        );
    }

    pub fn render_cursor(&self, terminal: &mut Frame<'_>, layout_rect: Rect) {
        let current_line = self.text.char_to_line(self.cursor_char_index);
        if current_line < self.scroll_y {
            return;
        }
        let max_lines_seen = self.scroll_y + layout_rect.height as usize - 2;
        if current_line > max_lines_seen {
            return;
        }

        let current_line_start = self.text.line_to_char(current_line);

        if self.cursor_char_index - current_line_start < self.scroll_x {
            return;
        }

        let cursor_y = current_line - self.scroll_y + 1;
        let mut cursor_x = (1 + visual_length_of_number(self.text.len_lines()) + 1) as usize;
        let mut to_remove = 0;
        for i in 0..(self.cursor_char_index - current_line_start) {
            let width =
                unicode_width::UnicodeWidthChar::width(self.text.char(current_line_start + i))
                    .unwrap_or(1);
            if i < self.scroll_x {
                to_remove += width;
            }
            cursor_x += width;
        }
        cursor_x -= to_remove;
        terminal.set_cursor(
            layout_rect.x + cursor_x as u16,
            layout_rect.y + cursor_y as u16,
        );
    }
}

static mut x: usize = 1;

impl Default for Window {
    fn default() -> Self {
        let y = unsafe {
            let t = x;
            x += 1;
            t
        };
        Window {
            ident: Some(format!("Window #{y}").to_string()),
            text: Rope::from_reader(BufReader::new(File::open("test_text.txt").unwrap())).unwrap(),
            scroll_x: 0,
            scroll_y: 0,
            cursor_char_index: 0,
            attached_file_path: Some("a".to_string()),
            modified: false,
            language: None,
            highlight_data: None,
            highlighter: Highlighter::new(),
        }
    }
}
