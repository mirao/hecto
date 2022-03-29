/// Generate keywords with their length so that length doesn't has to be computed with every searching of keyword in text
///
/// It returns e.g.:
/// ```
/// vec![("as", 2), ("break", 5)]
/// ```
/// for the input:
/// ```
/// &["as", "break"]
/// ```
fn generate_keywords_len(keywords: &[&str]) -> Vec<(String, usize)> {
    let mut keywords_with_len = Vec::new();

    for keyword in keywords {
        keywords_with_len.push(((*keyword).to_owned(), keyword.len()));
    }

    keywords_with_len
}

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
    multiline_comments: bool,
    primary_keywords: Vec<(String, usize)>,
    secondary_keywords: Vec<(String, usize)>,
}

impl HighlightingOptions {
    pub fn numbers(&self) -> bool {
        self.numbers
    }
    pub fn strings(&self) -> bool {
        self.strings
    }
    pub fn characters(&self) -> bool {
        self.characters
    }
    pub fn comments(&self) -> bool {
        self.comments
    }
    pub fn multiline_comments(&self) -> bool {
        self.multiline_comments
    }
    pub fn primary_keywords(&self) -> &Vec<(String, usize)> {
        &self.primary_keywords
    }
    pub fn secondary_keywords(&self) -> &Vec<(String, usize)> {
        &self.secondary_keywords
    }
}

pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn highlighting_options(&self) -> &HighlightingOptions {
        &self.hl_opts
    }

    pub fn from(file_name: &str) -> Self {
        if file_name
            .rsplit('.')
            .next()
            .map(|ext| ext.eq_ignore_ascii_case("rs"))
            == Some(true)
        {
            return Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiline_comments: true,
                    primary_keywords: generate_keywords_len(&[
                        "as", "break", "const", "continue", "crate", "else", "enum", "extern",
                        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
                        "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                        "super", "trait", "true", "type", "unsafe", "use", "where", "while", "dyn",
                        "abstract", "become", "box", "do", "final", "macro", "override", "priv",
                        "typeof", "unsized", "virtual", "yield", "async", "await", "try",
                    ]),
                    secondary_keywords: generate_keywords_len(&[
                        "bool", "char", "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32",
                        "u64", "usize", "f32", "f64",
                    ]),
                },
            };
        }
        Self::default()
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            hl_opts: HighlightingOptions::default(),
        }
    }
}
