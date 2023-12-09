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
