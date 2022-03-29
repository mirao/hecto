use crate::highlighting;
use crate::HighlightingOptions;
use crate::SearchDirection;

use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    pub is_highlighted: bool,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            is_highlighted: false,
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
    pub(crate) fn split(&mut self, at: usize) -> Self {
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
            is_highlighted: false,
            len: splitted_length,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if query.is_empty() {
            return None;
        }

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
        if let Some(matching_byte_index_unwrapped) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in substring.grapheme_indices(true).enumerate() {
                if matching_byte_index_unwrapped == byte_index {
                    #[allow(clippy::integer_arithmetic)]
                    return Some(start + grapheme_index);
                }
            }
        }
        None
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn highlight(
        &mut self,
        opts: &HighlightingOptions,
        word: &Option<String>,
        mut start_with_comment: bool,
    ) -> bool {
        let row = self.string.clone();
        let graphemes = row.graphemes(true).collect::<Vec<&str>>();

        if self.is_highlighted && word.is_none() {
            if let Some(hl_type) = self.highlighting.last() {
                return *hl_type == highlighting::Type::MultilineComment
                    && self.len() > 1
                    && if let Some(grapheme_asterisk) = graphemes.get(self.len() - 2) {
                        if grapheme_asterisk.contains('*') {
                            if let Some(grapheme_slash) = graphemes.get(self.len() - 1) {
                                !grapheme_slash.contains('/')
                            } else {
                                true
                            }
                        } else {
                            true
                        }
                    } else {
                        true
                    };
            }
        }

        self.highlighting = Vec::new();
        let mut index = 0;

        #[allow(clippy::shadow_unrelated)]
        while let Some(grapheme) = graphemes.get(index) {
            let is_multiline_comment_present;
            (is_multiline_comment_present, start_with_comment) = self.highlight_multiline_comment(
                &mut index,
                opts,
                start_with_comment,
                grapheme,
                &graphemes,
            );
            if is_multiline_comment_present {
                continue;
            }

            if self.highlight_char(&mut index, opts, grapheme, &graphemes)
                || self.highlight_comment(&mut index, opts, grapheme, &graphemes)
                || self.highlight_primary_keywords(&mut index, opts, &graphemes)
                || self.highlight_secondary_keywords(&mut index, opts, &graphemes)
                || self.highlight_string(&mut index, opts, grapheme, &graphemes)
                || self.highlight_number(&mut index, opts, grapheme, &graphemes)
            {
                continue;
            }
            self.highlighting.push(highlighting::Type::None);
            index = index.saturating_add(1);
        }

        self.highlight_match(word);
        self.is_highlighted = true;
        start_with_comment
    }

    /// Find and highlight all search matches on current line
    fn highlight_match(&mut self, word: &Option<String>) {
        if let Some(ref word_unwrapped) = *word {
            if word_unwrapped.is_empty() {
                return;
            }
            let mut index = 0;
            while let Some(search_match) =
                self.find(word_unwrapped, index, SearchDirection::Forward)
            {
                if let Some(next_index) =
                    search_match.checked_add(word_unwrapped.graphemes(true).count())
                {
                    #[allow(clippy::indexing_slicing)]
                    for i in search_match..next_index {
                        self.highlighting[i] = highlighting::Type::Match;
                    }
                    index = next_index;
                } else {
                    break;
                }
            }
        }
    }

    fn highlight_str(
        &mut self,
        index: &mut usize,
        substring: &str,
        graphemes: &[&str],
        hl_type: highlighting::Type,
    ) -> bool {
        if substring.is_empty() {
            return false;
        }
        for (substring_index, grapheme) in substring.graphemes(true).enumerate() {
            if let Some(next_char) = graphemes.get(index.saturating_add(substring_index)) {
                if *next_char != grapheme {
                    return false;
                }
            } else {
                return false;
            }
        }
        for _ in 0..substring.len() {
            self.highlighting.push(hl_type);
            *index = index.saturating_add(1);
        }
        true
    }

    fn highlight_keywords(
        &mut self,
        index: &mut usize,
        graphemes: &[&str],
        keywords: &[(String, usize)],
        hl_type: highlighting::Type,
    ) -> bool {
        if *index > 0 {
            if let Some(prev_grapheme) = graphemes.get(index.saturating_sub(1)) {
                if !is_separator(prev_grapheme) {
                    return false;
                }
            }
        }
        for word in keywords {
            // Originally (when keywords was an array of strings only) I used `let word_len = word.graphemes(true).count()`, but it causes very slow search. Moreover it's useless as keywords are ascii only
            let word_len = word.1;
            if *index < self.len().saturating_sub(word_len) {
                if let Some(next_grapheme) = graphemes.get(index.saturating_add(word_len)) {
                    if !is_separator(next_grapheme) {
                        continue;
                    }
                }
            }
            if self.highlight_str(index, &word.0, graphemes, hl_type) {
                return true;
            }
        }
        false
    }

    fn highlight_primary_keywords(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        graphemes: &[&str],
    ) -> bool {
        self.highlight_keywords(
            index,
            graphemes,
            opts.primary_keywords(),
            highlighting::Type::PrimaryKeywords,
        )
    }
    fn highlight_secondary_keywords(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        graphemes: &[&str],
    ) -> bool {
        self.highlight_keywords(
            index,
            graphemes,
            opts.secondary_keywords(),
            highlighting::Type::SecondaryKeywords,
        )
    }

    fn highlight_char(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        grapheme: &str,
        graphemes: &[&str],
    ) -> bool {
        if opts.characters() && grapheme.contains('\'') {
            if let Some(next_grapheme) = graphemes.get(index.saturating_add(1)) {
                let closing_index = if next_grapheme.contains('\\') {
                    index.saturating_add(3)
                } else {
                    index.saturating_add(2)
                };
                if let Some(closing_grapheme) = graphemes.get(closing_index) {
                    if closing_grapheme.contains('\'') {
                        for _ in 0..=closing_index.saturating_sub(*index) {
                            self.highlighting.push(highlighting::Type::Character);
                            *index = index.saturating_add(1);
                        }
                        return true;
                    }
                }
            };
        }
        false
    }

    fn highlight_comment(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        grapheme: &str,
        graphemes: &[&str],
    ) -> bool {
        if opts.comments() && grapheme.contains('/') {
            if let Some(next_grapheme) = graphemes.get(index.saturating_add(1)) {
                if next_grapheme.contains('/') {
                    for _ in *index..self.len() {
                        self.highlighting.push(highlighting::Type::Comment);
                        *index = index.saturating_add(1);
                    }
                    return true;
                }
            };
        }
        false
    }

    fn highlight_multiline_comment(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        mut start_with_comment: bool,
        grapheme: &str,
        graphemes: &[&str],
    ) -> (bool, bool) {
        if start_with_comment {
            *index = self.len();
            for (index_iter, grapheme_iter) in graphemes.iter().enumerate() {
                if grapheme_iter.contains('*') {
                    if let Some(closing_slash) = graphemes.get(index_iter.saturating_add(1)) {
                        if closing_slash.contains('/') {
                            start_with_comment = false;
                            *index = index_iter.saturating_add(2);
                            break;
                        }
                    }
                }
            }
            for _ in 0..*index {
                self.highlighting.push(highlighting::Type::MultilineComment);
            }
            return (true, start_with_comment);
        }

        start_with_comment = true;
        if opts.multiline_comments() && grapheme.contains('/') {
            if let Some(next_grapheme) = graphemes.get(index.saturating_add(1)) {
                if next_grapheme.contains('*') {
                    let mut closing_index = self.len();
                    for (index_iter, grapheme_iter) in
                        graphemes.iter().skip(index.saturating_add(2)).enumerate()
                    {
                        if grapheme_iter.contains('*') {
                            if let Some(closing_slash) =
                                graphemes.get(index.saturating_add(index_iter.saturating_add(3)))
                            {
                                if closing_slash.contains('/') {
                                    closing_index =
                                        index.saturating_add(index_iter.saturating_add(4));
                                    start_with_comment = false;
                                    break;
                                }
                            }
                        }
                    }
                    for _ in *index..closing_index {
                        self.highlighting.push(highlighting::Type::MultilineComment);
                    }
                    *index = closing_index;
                    return (true, start_with_comment);
                }
            }
        };
        (false, false)
    }

    fn highlight_string(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        grapheme: &str,
        graphemes: &[&str],
    ) -> bool {
        if opts.strings() & grapheme.contains('"') {
            loop {
                self.highlighting.push(highlighting::Type::String);
                *index = index.saturating_add(1);
                if let Some(next_grapheme) = graphemes.get(*index) {
                    // '\a' or '\"' is a part of string
                    if next_grapheme.contains('\\') {
                        self.highlighting.push(highlighting::Type::String);
                        *index = index.saturating_add(1);
                    } else if next_grapheme.contains('"') {
                        break;
                    }
                } else {
                    break;
                }
            }
            self.highlighting.push(highlighting::Type::String);
            *index = index.saturating_add(1);
            return true;
        }
        false
    }

    fn highlight_number(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        grapheme: &str,
        graphemes: &[&str],
    ) -> bool {
        if opts.numbers() && grapheme.chars().any(|c| c.is_ascii_digit()) {
            if *index > 0 {
                if let Some(prev_grapheme) = graphemes.get(index.saturating_sub(1)) {
                    if !is_separator(prev_grapheme) {
                        return false;
                    }
                }
            }
            loop {
                self.highlighting.push(highlighting::Type::Number);
                *index = index.saturating_add(1);
                if let Some(next_grapheme) = graphemes.get(*index) {
                    if next_grapheme
                        .chars()
                        .any(|c| c != '.' && !c.is_ascii_digit())
                    {
                        break;
                    }
                } else {
                    break;
                }
            }
            return true;
        }
        false
    }

    pub fn get_string(&self) -> String {
        self.string.clone()
    }
}

fn is_separator(grapheme: &str) -> bool {
    grapheme
        .chars()
        .any(|c| c.is_ascii_punctuation() || c.is_ascii_whitespace())
}
