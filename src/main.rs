use std::{io::{Error, BufReader}, fs::File};

use ropey::Rope;

#[derive(Debug)]
struct Editor {
    text: Rope,
}

#[derive(Copy, Clone, Debug)]
struct Line(usize);
#[derive(Copy, Clone, Debug)]
struct Char(usize);

impl Editor {

    pub fn insert_at(&mut self, line: Line, char_index: Char, to_insert: &str) -> bool {
        if line.0 >= self.text.len_lines() {
            return false;
        }

        let start_idx = self.combine_line_and_char(line, char_index);
        self.text.insert(start_idx, to_insert);
        true
    }

    pub fn insert_new_line(&mut self, line: Line, char_index: Char, amount_of_lines: usize) {
        let start_index = self.combine_line_and_char(line, char_index);
        for _ in 0..amount_of_lines {
            self.text.insert_char(start_index, '\n');
        }
    }

    pub fn append_new_lines(&mut self, amount_of_lines: usize) {
        for _ in 0..amount_of_lines {
            self.text.insert_char(self.text.len_chars(), '\n');
        }
    }

    pub fn append_at_line(&mut self, line: Line, text: &str) {
        let line = line.0;
        let len_lines = self.text.len_lines();

        let idx = if line == len_lines - 1 {
            self.text.line(line).len_chars()
        } else {
            self.text.line(line).len_chars() - 1
        };

        self.insert_at(Line(line), Char(idx), text);
    }

    fn combine_line_and_char(&self, line: Line, char: Char) -> usize {
        self.text.line_to_char(line.0) + char.0
    }

}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn editor_test() {
        let mut editor = Editor {
            text: Rope::from_str(""),
        };
        text_assert_eq(&editor, "");
        editor.insert_at(Line(0), Char(0), "Hello, I have been inserted!");
        text_assert_eq(&editor, "Hello, I have been inserted!");
        editor.insert_new_line(Line(0), Char(0), 1);
        text_assert_eq(&editor, "\nHello, I have been inserted!");
        editor.insert_new_line(Line(1), Char(2), 2);
        text_assert_eq(&editor, "\nHe\n\nllo, I have been inserted!");
        editor.append_new_lines(3);
        text_assert_eq(&editor, "\nHe\n\nllo, I have been inserted!\n\n\n");
        editor.insert_at(Line(0), Char(0), "Inserted At The Beginning");
        text_assert_eq(&editor, "Inserted At The Beginning\nHe\n\nllo, I have been inserted!\n\n\n");
        editor.append_at_line(Line(0), " and this was appended");
        text_assert_eq(&editor, "Inserted At The Beginning and this was appended\nHe\n\nllo, I have been inserted!\n\n\n");
    }

    #[test]
    fn append_empty_text() {
        let mut editor = Editor {
            text: Rope::from_str(""),
        };

        text_assert_eq(&editor, "");
        editor.append_at_line(Line(0), "Not empty anymore");
        text_assert_eq(&editor, "Not empty anymore");
        editor.append_at_line(Line(0), " more text");
        text_assert_eq(&editor, "Not empty anymore more text");
        editor.append_new_lines(1);
        text_assert_eq(&editor, "Not empty anymore more text\n");
    }

    fn text_assert_eq(actual: &Editor, expected: &str) {
        assert_eq!(actual.text.to_string(), expected);
    }

}

fn main() -> Result<(), Error> {
    let mut test_text = Editor { text: Rope::from_reader(BufReader::new(File::open("test_text.txt")?))? };
    test_text.insert_at(Line(0), Char(2), "Hallo ich wurde inserted!");
    test_text.append_new_lines(1);
    test_text.append_new_lines(2);
    test_text.insert_new_line(Line(0), Char(5), 1);
    dbg!(test_text);
    Ok(())
}
