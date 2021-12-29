use termion::color;

pub enum Type {
    None,
    Number,
}

impl Type {
    pub fn to_color(&self) -> color::Rgb {
        match *self {
            Type::Number => color::Rgb(220, 163, 163),
            Type::None => color::Rgb(255, 255, 255),
        }
    }
}
