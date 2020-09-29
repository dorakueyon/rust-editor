use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::*;

use std::fmt::Debug;

use termion::screen::AlternateScreen;

const KILO_VERSION: &str = "1.0";

pub struct Viewer {
  stdout: AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
  cursor_x: u16,
  cursor_y: u16,
  window_size_col: u16,
  window_size_row: u16,
  editor_lines: Vec<EditorLine>,
}

struct EditorLine {
  line: String,
}

impl Viewer {
  fn enable_raw_mode() -> AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>> {
    let stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    stdout
  }

  fn get_window_size() -> (u16, u16) {
    let (col, row) = termion::terminal_size().unwrap();
    (col, row)
  }

  fn new() -> Self {
    let stdout = Viewer::enable_raw_mode();
    let (window_size_col, window_size_row) = Viewer::get_window_size();
    let cursor_x = 1;
    let cursor_y = 1;

    Self {
      stdout,
      cursor_x,
      cursor_y,
      window_size_col,
      window_size_row,
      editor_lines: vec![],
    }
  }

  fn editor_open(&mut self) {
    let editor_lines = vec![
      EditorLine {
        line: String::from("hello world"),
      },
      EditorLine {
        line: String::from("hey cutie"),
      },
    ];
    self.editor_lines = editor_lines
  }

  fn saturated_add_x(&mut self, x: u16) {
    if self.cursor_x < self.window_size_col {
      let cursor_x = self.cursor_x + x;
      write!(
        self.stdout,
        "{}",
        cursor::Goto(self.cursor_x, self.cursor_y)
      );
    }
  }

  fn saturated_substract_x(&mut self, x: u16) {
    if 0 < self.cursor_x {
      let cursor_x = self.cursor_x - x;
      write!(
        self.stdout,
        "{}",
        cursor::Goto(self.cursor_x, self.cursor_y)
      );
    }
  }

  fn saturated_add_y(&mut self, y: u16) {
    if self.cursor_y < self.window_size_row {
      let cursor_y = self.cursor_y + y;
      write!(
        self.stdout,
        "{}",
        cursor::Goto(self.cursor_x, self.cursor_y)
      );
    }
  }

  fn saturated_substract_y(&mut self, y: u16) {
    if 0 < self.cursor_y {
      let cursor_x = self.cursor_y - y;
      write!(
        self.stdout,
        "{}",
        cursor::Goto(self.cursor_x, self.cursor_y)
      );
    }
  }

  fn editor_process_key_press(&mut self) {
    for c in stdin().keys() {
      //write!(self.stdout, "{:?}", c);
      self.stdout.flush().unwrap();
      match c {
        Ok(event::Key::Ctrl('c')) | Ok(event::Key::Ctrl('q')) => break,
        Ok(event::Key::Left) => {
          self.saturated_substract_x(1);
        }
        Ok(event::Key::Right) => {
          self.saturated_add_x(1);
        }
        Ok(event::Key::Up) => {
          self.saturated_substract_y(1);
        }
        Ok(event::Key::Down) => {
          self.saturated_add_y(1);
        }
        _ => {}
      }
    }
  }

  fn editor_refresh_screen(&mut self) {
    write!(self.stdout, "{}{}", clear::All, cursor::Hide).unwrap();
    write!(
      self.stdout,
      "{}",
      cursor::Goto(self.cursor_x, self.cursor_y)
    )
    .unwrap();
    self.editor_draw_rows();
    self.stdout.flush().unwrap();
  }

  fn get_welcome_line(&mut self) -> String {
    let welcom_message = format!("igc editor -- version {}", KILO_VERSION);
    let mut welcom_len = welcom_message.chars().count() as u16;
    if welcom_len > self.window_size_col {
      welcom_len = self.window_size_col;
    }

    let mut welcome_line = String::new();
    let mut padding = (self.window_size_col - welcom_len) / 2;
    welcome_line.push('~');
    padding = padding - 1;
    for _ in 0..padding {
      welcome_line.push(' ');
    }

    for i in 0..welcom_len {
      let c = welcom_message.chars().nth(i as usize).unwrap();
      welcome_line.push(c);
    }

    welcome_line
  }

  fn editor_draw_rows(&mut self) {
    for i in 0..self.window_size_row {
      eprintln!("{}", self.editor_lines.len());
      if i >= self.editor_lines.len() as u16 {
        if i == (self.window_size_row / 3) {
          let welcome_line = self.get_welcome_line();
          write!(self.stdout, "{}", welcome_line);
        } else {
          let line = format!("~ ");
          write!(self.stdout, "{}", line);
        }
      } else {
        let line = format!("{}", self.editor_lines[i as usize].line);
        write!(self.stdout, "{}", line);
      }

      if i < self.window_size_row - 1 {
        write!(self.stdout, "\r\n");
      }
    }
  }

  pub fn run_viwer() {
    let mut viewer = Viewer::new();

    viewer.editor_open();
    viewer.editor_refresh_screen();

    viewer.editor_process_key_press();

    write!(viewer.stdout, "{}", termion::cursor::Show).unwrap();
  }
}
