use termion::color;

#[derive(PartialEq)]
pub enum Type {
    Number, // Highlighted number
    Match,  // Highlighted match in search
    None,   // No highlighting
}

impl Type {
    pub fn to_color(&self) -> color::Rgb {
        match *self {
            Type::Number => color::Rgb(220, 163, 163),
            Type::Match => color::Rgb(38, 139, 210),
            Type::None => color::Rgb(255, 255, 255),
        }
    }
}
