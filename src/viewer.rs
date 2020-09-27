use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::*;

use termion::screen::AlternateScreen;

const KILO_VERSION: &str = "1.0";

pub struct Viewer {
  stdout: AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
  window_size_col: usize,
  window_size_row: usize,
}

impl Viewer {
  fn enable_raw_mode() -> AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>> {
    let stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    stdout
  }

  fn get_window_size() -> (usize, usize) {
    let (col, row) = termion::terminal_size().unwrap();
    (col as usize, row as usize)
  }

  fn new() -> Self {
    let stdout = Viewer::enable_raw_mode();
    let (window_size_col, window_size_row) = Viewer::get_window_size();
    Self {
      stdout,
      window_size_col,
      window_size_row,
    }
  }
  fn editor_process_key_press(&mut self) {
    for c in stdin().keys() {
      write!(self.stdout, "{:?}", c);
      self.stdout.flush().unwrap();
      match c {
        Ok(event::Key::Ctrl('c')) | Ok(event::Key::Ctrl('q')) => break,
        _ => {}
      }
    }
  }

  fn editor_refresh_screen(&mut self) {
    write!(self.stdout, "{}{}", clear::All, cursor::Hide).unwrap();
    write!(self.stdout, "{}", cursor::Goto(1, 1)).unwrap();
    self.editor_draw_rows();
    self.stdout.flush().unwrap();
  }

  fn editor_draw_rows(&mut self) {
    for i in 0..self.window_size_row {
      if i == (self.window_size_col / 3) {
        let welcom_message = format!("igc editor -- version {}", KILO_VERSION);
        let mut welcom_len = welcom_message.chars().count();
        if welcom_len > self.window_size_col {
          welcom_len = self.window_size_col;
        }
        for i in 0..welcom_len {
          let c = welcom_message.chars().nth(i).unwrap();
          write!(self.stdout, "{}", c);
        }
      } else {
        let line = format!("~ ");
        write!(self.stdout, "{}", line);
      }
      if i < self.window_size_row - 1 {
        write!(self.stdout, "\r\n");
      }
    }
  }

  pub fn run_viwer() {
    let mut viewer = Viewer::new();
    viewer.editor_refresh_screen();

    viewer.editor_process_key_press();

    write!(viewer.stdout, "{}", termion::cursor::Show).unwrap();
  }
}
