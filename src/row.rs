use crate::highlighting;
use crate::SearchDirection;

use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        let mut current_highlighting = &highlighting::Type::None;
        #[allow(clippy::integer_arithmetic)]
        for (index, grapheme) in self
            .string
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                let highlighting_type = self
                    .highlighting
                    .get(index)
                    .unwrap_or(&highlighting::Type::None);
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    let start_highlight =
                        format!("{}", termion::color::Fg(highlighting_type.to_color()));
                    result.push_str(&start_highlight);
                }
                if c == '\t' {
                    result.push(' ');
                } else {
                    result.push(c);
                }
            }
        }
        let end_highlight = format!("{}", termion::color::Fg(color::Reset));
        result.push_str(&end_highlight);
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            // Insert character to the end of line
            self.string.push(c);
        } else {
            // Insert character in the middle of line
            let mut result: String = String::new();
            for (index, grapheme) in self.string.graphemes(true).enumerate() {
                if index == at {
                    result.push(c);
                }
                result.push_str(grapheme);
            }
            self.string = result;
        }

        // Prevents counting of emoji flag sequence, e.g. 🇨🇿 as two chars (🇨 and 🇿) instead of expected one char
        // Double counting causes empty chars on line
        // More info about emoji flags at https://en.wikipedia.org/wiki/Regional_indicator_symbol
        if self.string.graphemes(true).count() > self.len() {
            self.len += 1;
        }
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }

        let mut result: String = String::new();
        for (index, grapheme) in self.string.graphemes(true).enumerate() {
            if index != at {
                result.push_str(grapheme);
            }
        }
        self.len -= 1;
        self.string = result;
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn split(&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut length = 0;
        let mut splitted_row: String = String::new();
        for (index, grapheme) in self.string.graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_row.push_str(grapheme);
            }
        }

        let splitted_length = self.len - length;
        self.string = row;
        self.len = length;
        Self {
            string: splitted_row,
            highlighting: Vec::new(),
            len: splitted_length,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        let start = if direction == SearchDirection::Forward {
            at
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            at
        };
        #[allow(clippy::integer_arithmetic)]
        let substring: String = self
            .string
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };
        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in substring.grapheme_indices(true).enumerate() {
                if matching_byte_index == byte_index {
                    #[allow(clippy::integer_arithmetic)]
                    return Some(start + grapheme_index);
                }
            }
        }
        None
    }

    pub fn highlight(&mut self) {
        let mut highlighting = Vec::new();
        for grapheme in self.string.graphemes(true) {
            if grapheme.chars().any(|c| c.is_ascii_digit()) {
                highlighting.push(highlighting::Type::Number);
            } else {
                highlighting.push(highlighting::Type::None);
            }
        }
        self.highlighting = highlighting;
    }

    pub fn get_string(&self) -> String {
        self.string.clone()
    }
}
