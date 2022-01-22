use std::{
    fs,
    io::{self, Write},
};

use crate::SearchDirection;
use crate::{Position, Row};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
}

impl Document {
    /// # Errors
    ///
    /// Will return `Err` if `filename` does not exist or the user does not have
    /// permission to read it.
    pub fn open(filename: &str) -> io::Result<Self> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            let mut row = Row::from(value);
            row.highlight(None);
            rows.push(row);
        }

        // Append last empty line if exists in document
        // Note that previous using of `contents.lines()` ignores ending newline
        if contents.ends_with('\n') {
            rows.push(Row::from(""));
        }

        Ok(Self {
            rows,
            file_name: Some(filename.to_owned()),
            dirty: false,
        })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn row_len(&self, y: usize) -> usize {
        self.row(y).map_or(0, Row::len)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    fn insert_newline(&mut self, at: &Position) {
        if self.is_empty() {
            // Empty document will have two new lines
            self.rows.push(Row::default());
            self.rows.push(Row::default());
        } else {
            #[allow(clippy::indexing_slicing)]
            let current_row = &mut self.rows[at.y];
            let mut new_row = current_row.split(at.x);
            current_row.highlight(None);
            new_row.highlight(None);
            #[allow(clippy::integer_arithmetic)]
            self.rows.insert(at.y + 1, new_row);
        }
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        self.dirty = true;

        if c == '\n' {
            self.insert_newline(at);
            return;
        }

        if self.is_empty() {
            // Insert char to new line
            let mut row = Row::default();
            row.insert(0, c);
            row.highlight(None);
            self.rows.push(row);
        } else {
            // Insert char inside existing line
            #[allow(clippy::indexing_slicing)]
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
            row.highlight(None);
        }
    }

    #[allow(clippy::integer_arithmetic, clippy::indexing_slicing)]
    pub fn delete(&mut self, at: &Position) {
        if self.is_empty()
            || at.x == self.row_len(at.y) && self.row(at.y.saturating_add(1)).is_none()
        {
            return;
        }

        self.dirty = true;

        let len = self.len();
        if at.x == self.row_len(at.y) && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
            row.highlight(None);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
            row.highlight(None);
        }
    }

    /// # Errors
    ///
    /// Fails if file cannot be open in write mode or content cannot be written
    pub fn save(&mut self) -> io::Result<()> {
        if let Some(ref file_name) = self.file_name {
            let mut file = fs::File::create(file_name)?;
            for (i, row) in self.rows.iter().enumerate() {
                file.write_all(row.as_bytes())?;
                #[allow(clippy::integer_arithmetic)]
                if i < self.len() - 1 {
                    file.write_all(b"\n")?;
                }
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[allow(clippy::indexing_slicing)]
    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        let mut position = Position { x: at.x, y: at.y };

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };
        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }
    pub fn highlight(&mut self, word: Option<&str>) {
        for row in &mut self.rows {
            row.highlight(word);
        }
    }
}
