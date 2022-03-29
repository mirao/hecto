use termion::color;

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    Number,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
    Match, // Highlight match in search
    None,  // No highlighting
}

impl Type {
    pub fn to_color(self) -> color::Rgb {
        match self {
            Type::Number => color::Rgb(220, 163, 163),
            Type::String => color::Rgb(211, 54, 130),
            Type::Character => color::Rgb(108, 113, 196),
            Type::Comment | Type::MultilineComment => color::Rgb(0x67, 0x95, 0x4f),
            Type::PrimaryKeywords => color::Rgb(181, 137, 0),
            Type::SecondaryKeywords => color::Rgb(42, 161, 152),
            Type::Match => color::Rgb(38, 139, 210),
            Type::None => color::Rgb(255, 255, 255),
        }
    }
}
