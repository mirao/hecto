use crate::Document;
use crate::Row;
use crate::Terminal;

use std::cmp;
use std::env;
use std::io;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;
use unicode_segmentation::UnicodeSegmentation;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

#[non_exhaustive]
#[derive(PartialEq, Clone, Copy)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone)]
#[non_exhaustive]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(&error);
            }
        }
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status =
            String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");
        let document = if let Some(file_name) = args.get(1) {
            let doc = Document::open(file_name);
            if let Ok(doc) = doc {
                doc
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        #[allow(clippy::expect_used)]
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
        }
    }

    fn refresh_screen(&self) -> io::Result<()> {
        Terminal::cursor_hide();
        Terminal::set_cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::set_cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    #[allow(clippy::integer_division)]
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        #[allow(clippy::integer_division)]
        let padding = width.saturating_sub(len.saturating_add(1)) / 2;
        let spaces = " ".repeat(padding);
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn process_keypress(&mut self) -> io::Result<()> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => {
                #[allow(clippy::integer_arithmetic)]
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return Ok(());
                }
                self.should_quit = true;
            }
            Key::Ctrl('s') => self.save(),
            Key::Ctrl('f') => self.search(),
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                if let Some(row) = self.document.row(self.cursor_position.y) {
                    let mut move_cursor = false;
                    if let Some(grapheme) =
                        row.get_string().graphemes(true).nth(self.cursor_position.x)
                    {
                        // Move cursor for single unicode characters and do not move cursor in case of finished emoji flag sequence, e.g. 🇨🇿 (two unicode characters in one)
                        if grapheme == c.to_string() {
                            move_cursor = true;
                        }
                    } else {
                        // ENTER is pressed
                        move_cursor = true;
                    }
                    if move_cursor {
                        self.move_cursor(Key::Right);
                    }
                }
            }
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(pressed_key),
            _ => {}
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_owned());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_owned());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_owned());
        }
    }

    fn search(&mut self) {
        let mut direction = SearchDirection::Forward;
        self.prompt(
            "Search (ESC to cancel, Arrows to navigate): ",
            |editor, key, query| {
                let mut moved = false;
                match key {
                    Key::Right | Key::Down => {
                        direction = SearchDirection::Forward;
                        editor.move_cursor(Key::Right);
                        moved = true;
                    }
                    Key::Left | Key::Up => direction = SearchDirection::Backward,
                    _ => direction = SearchDirection::Forward,
                }
                if let Some(position) =
                    editor
                        .document
                        .find(query, &editor.cursor_position, direction)
                {
                    editor.cursor_position = position;
                    editor.scroll();
                } else if moved {
                    editor.move_cursor(Key::Left);
                }
                editor.document.highlight(Some(query));
            },
        )
        .unwrap_or(None);
        self.document.highlight(None);
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let last_line_pos = self.document.len().saturating_sub(1);
        let width = self.document.row_len(y);

        match key {
            Key::Up => {
                y = y.saturating_sub(1);
            }
            Key::Down =>
            {
                #[allow(clippy::integer_arithmetic)]
                if y < last_line_pos {
                    y += 1;
                }
            }
            Key::Left =>
            {
                #[allow(clippy::integer_arithmetic)]
                if x > 0 {
                    x = x.saturating_sub(1);
                } else if y > 0 {
                    y -= 1;
                    x = self.document.row_len(y);
                }
            }
            Key::Right =>
            {
                #[allow(clippy::integer_arithmetic)]
                if x < width {
                    x += 1;
                } else if y < last_line_pos {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => y = y.saturating_sub(terminal_height),
            Key::PageDown => y = cmp::min(y.saturating_add(terminal_height), last_line_pos),
            Key::Home => x = 0,
            Key::End => x = width,
            _ => {}
        };

        // When moving cursor up/down
        // and X position of previous line is greater than current line width, move cursor to the end of line
        if let Key::Up | Key::Down | Key::PageDown | Key::PageUp = key {
            x = cmp::min(x, self.document.row_len(y));
        }

        self.cursor_position = Position { x, y }
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;

        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };

        let mut file_name = "[No Name]".to_owned();
        if let Some(ref name) = self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }

        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );
        let line_indicator = format!(
            "Ln {}, Col {}",
            self.cursor_position.y.saturating_add(1),
            self.cursor_position.x.saturating_add(1),
        );
        #[allow(clippy::integer_arithmetic)]
        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: FnMut(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => {
                    let graphemes_cnt = result.graphemes(true).count();
                    result = result
                        .graphemes(true)
                        .take(graphemes_cnt.saturating_sub(1))
                        .collect();
                }
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
}

#[allow(clippy::panic)]
fn die(e: &std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
