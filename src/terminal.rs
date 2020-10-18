use crate::Position;

use std::io::{self, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::*;

const STATUS_LINE_LENGTH: u16 = 2;

pub struct Terminal {
    pub window_size_width: u16,  // TODO: remove pub
    pub window_size_height: u16, // TODO: remove pub
    _stdout: AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
}

impl Terminal {
    pub fn default() -> Self {
        let (window_size_width, mut window_size_height) = Terminal::get_window_size();
        window_size_height = window_size_height - STATUS_LINE_LENGTH;
        Terminal {
            window_size_height,
            window_size_width,
            _stdout: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
        }
    }

    fn get_window_size() -> (u16, u16) {
        let (width, height) = termion::terminal_size().unwrap();
        (width, height)
    }

    pub fn clear_screen() {
        print!("{}", termion::clear::All)
    }

    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide)
    }

    pub fn cursor_show() {
        print!("{}", termion::cursor::Show)
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
        let x = position.x.saturating_add(1);
        let y = position.y.saturating_add(1);
        print!(
            "{}{}",
            cursor::BlinkingBar,
            cursor::Goto(x as u16, y as u16)
        );
    }

    pub fn read_key() -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = std::io::stdin().lock().keys().next() {
                return key;
            }
        }
    }

    pub fn flush() -> Result<(), std::io::Error> {
        std::io::stdout().flush()
    }
}
