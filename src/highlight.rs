use termion::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Highlight {
    Normal,
    Number,
    Match,
    String,
    Comment,
    MultiComment,
    Keyword1,
    Keyword2,
}

impl Highlight {
    pub fn editor_syntax_to_color(self) -> termion::color::AnsiValue {
        match self {
            Highlight::Normal => color::AnsiValue(7), // White
            Highlight::Number => color::AnsiValue(1), // Red
            Highlight::Match => color::AnsiValue(4),  // Blue
            Highlight::String => color::AnsiValue(5), // Magenta
            Highlight::Comment => color::AnsiValue(6), // Cyan
            Highlight::MultiComment => color::AnsiValue(6), // Cyan
            Highlight::Keyword1 => color::AnsiValue(2), // Green
            Highlight::Keyword2 => color::AnsiValue(3), // Yellow
        }
    }
}
