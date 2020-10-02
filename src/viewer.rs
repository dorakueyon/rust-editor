use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::*;

use chrono::{DateTime, Duration, Utc};
use std::fmt::Debug;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

use termion::screen::AlternateScreen;

const KILO_VERSION: &str = "1.0";
const STATUS_LINE_LENGTH: u16 = 2;

pub struct Viewer {
    stdout: AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
    cursor_x: u16,
    render_x: u16,
    cursor_y: u16,
    row_offset: u16,
    column_offset: u16,
    window_size_col: u16,
    window_size_row: u16,
    editor_lines: Vec<EditorLine>,
    file_name: Option<String>,
    status_message: String,
    status_message_time: DateTime<Utc>,
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
        let (window_size_col, mut window_size_row) = Viewer::get_window_size();

        window_size_row = window_size_row - STATUS_LINE_LENGTH;
        let cursor_x = 0;
        let cursor_y = 0;
        let render_x = 0;
        let row_offset = 0;
        let column_offset = 0;
        let status_message = String::new();
        let status_message_time = Utc::now();

        Self {
            stdout,
            cursor_x,
            cursor_y,
            render_x,
            row_offset,
            column_offset,
            window_size_col,
            window_size_row,
            editor_lines: vec![],
            file_name: None,
            status_message,
            status_message_time,
        }
    }

    fn editor_open(&mut self, file_name: &str) {
        let file = match File::open(file_name) {
            Err(why) => panic!("couldn't open {}: {}", file_name, why),
            Ok(file) => file,
        };

        let mut editor_lines = vec![];
        for line in BufReader::new(file).lines() {
            match line {
                Ok(s) => editor_lines.push(EditorLine { line: s }),
                Err(_) => {}
            }
        }
        self.editor_lines = editor_lines;

        self.file_name = Some(String::from(file_name));
    }

    fn saturated_add_x(&mut self) {
        if self.cursor_x < self.get_current_row_length() {
            self.cursor_x = self.cursor_x + 1;
        } else {
            if self.cursor_y + 1 < self.get_editor_line_length() {
                self.saturated_add_y();
                self.cursor_x = 0;
            }
        }
    }

    fn saturated_substract_x(&mut self) {
        if 0 < self.cursor_x {
            self.cursor_x = self.cursor_x - 1;
        } else {
            if 0 < self.cursor_y {
                self.saturated_substract_y();
                self.cursor_x = self.get_current_row_length();
            }
        }
    }

    fn saturated_add_y(&mut self) {
        if self.cursor_y + 1 < self.get_editor_line_length() {
            self.cursor_y = self.cursor_y + 1;
            if (self.cursor_x) > self.get_current_row_length() {
                self.cursor_x = self.get_current_row_length()
            }
        }
    }

    fn saturated_substract_y(&mut self) {
        if 0 < self.cursor_y {
            self.cursor_y = self.cursor_y - 1;
            if (self.cursor_x) > self.get_current_row_length() {
                self.cursor_x = self.get_current_row_length()
            }
        }
    }

    fn editor_row_insert_char(&mut self, c: char) {
        let mut new_line = String::new();
        if self.cursor_x == self.get_current_row_length() {
            new_line = format!("{}{}", self.editor_lines[self.cursor_y as usize].line, c)
        } else {
            for (i, c_existed) in self.editor_lines[self.cursor_y as usize]
                .line
                .chars()
                .enumerate()
            {
                if i == self.cursor_x as usize {
                    new_line.push(c)
                }
                new_line.push(c_existed)
            }
        }
        self.editor_lines[self.cursor_y as usize].line = new_line;
        self.saturated_add_x();
    }

    fn editor_insert_char(&mut self, c: char) {
        self.editor_row_insert_char(c)
    }

    fn editor_save(&mut self) {
        match &self.file_name {
            None => return,
            Some(s) => {
                eprintln!("{}", s);
                match File::create(s) {
                    Ok(mut f) => {
                        for i in 0..self.get_editor_line_length() {
                            let line = &self.editor_lines[i as usize].line;
                            f.write_all(line.as_bytes()).unwrap();
                            f.write_all(b"\r\n").unwrap();
                        }
                        self.set_status_message(format!(
                            "{} bytes written to disk",
                            self.get_editor_buffer_length()
                        ));
                    }

                    Err(e) => self.set_status_message(format!("Can't save! I/O error: {}", e)),
                }
            }
        }
    }

    fn editor_process_key_press(&mut self) {
        for c in stdin().keys() {
            dbg!(&c);
            self.stdout.flush().unwrap();
            match c {
                Ok(event::Key::Ctrl('c')) | Ok(event::Key::Ctrl('q')) => break,
                Ok(event::Key::Delete) => {} //TODO
                Ok(event::Key::Backspace) | Ok(event::Key::Ctrl('h')) => {} //TODO
                Ok(event::Key::Ctrl('s')) => self.editor_save(),
                Ok(event::Key::Left) => {
                    self.saturated_substract_x();
                }
                Ok(event::Key::Right) => {
                    self.saturated_add_x();
                }
                Ok(event::Key::Up) => {
                    self.saturated_substract_y();
                }
                Ok(event::Key::Down) => {
                    self.saturated_add_y();
                }
                Ok(event::Key::Char(c)) => {
                    if c == '\n' {
                        // enter
                        eprintln!("enter pressed");
                    } else {
                        self.editor_insert_char(c)
                    }
                }
                _ => {}
            }
            self.editor_refresh_screen()
        }
    }

    fn editor_row_cx2rx(&mut self) -> u16 {
        self.cursor_x
    }

    fn editor_refresh_screen(&mut self) {
        self.editor_scroll();
        write!(self.stdout, "{}", clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Goto(1, 1)).unwrap();

        self.editor_draw_rows();
        self.editor_draw_status_bar();
        self.editor_draw_message_bar();

        eprintln!(
            "cursor goto {}: {}. row_offset: {}. editor_line: {}",
            self.render_x,
            self.cursor_y,
            self.row_offset,
            self.editor_lines.len()
        );
        write!(
            self.stdout,
            "{}",
            cursor::Goto(
                self.render_x + 1 - self.column_offset,
                self.cursor_y + 1 - self.row_offset
            )
        )
        .unwrap();

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

    fn get_editor_line_length(&self) -> u16 {
        self.editor_lines.len() as u16
    }

    fn get_editor_buffer_length(&self) -> u16 {
        let mut sum = 0;
        for line in &self.editor_lines {
            sum = sum + line.line.as_bytes().len()
        }
        sum as u16
    }

    fn get_current_row_length(&self) -> u16 {
        self.editor_lines[self.cursor_y as usize]
            .line
            .chars()
            .count() as u16
    }

    fn set_status_message(&mut self, status_massage: String) {
        self.status_message = status_massage;
        self.status_message_time = Utc::now()
    }

    fn editor_draw_status_bar(&mut self) {
        let file_name_limit_char_length = 20 as usize;
        let mut display_file_name = String::new();
        match &self.file_name {
            Some(s) => {
                for (i, c) in s.chars().enumerate() {
                    if i < file_name_limit_char_length {
                        display_file_name.push(c);
                    }
                }
                display_file_name = s.to_string()
            }
            None => display_file_name = String::from("[No Name]"),
        }
        let mut status = format!(
            "{} - {} lines",
            display_file_name,
            self.get_editor_line_length()
        );
        let right_status = format!("{}/{}", self.cursor_y + 1, self.get_editor_line_length());
        if status.chars().count() + right_status.chars().count() < self.window_size_col as usize {
            for _ in
                status.chars().count()..self.window_size_col as usize - right_status.chars().count()
            {
                status.push(' ')
            }
        }
        let status_line = format!("{}{}", status, right_status);

        write!(
            self.stdout,
            "{}{}{}{}",
            color::Bg(color::LightMagenta),
            color::Fg(color::Black),
            status_line,
            style::Reset
        )
        .unwrap();
        write!(self.stdout, "\r\n").unwrap();
    }

    fn editor_draw_message_bar(&mut self) {
        let mut message_line = String::new();
        if self.status_message_time + Duration::seconds(5) < Utc::now() {
            return;
        }
        for (i, c) in self.status_message.chars().enumerate() {
            if i < self.window_size_col as usize {
                message_line.push(c)
            }
        }

        write!(
            self.stdout,
            "{}{}{}{}",
            color::Bg(color::LightMagenta),
            color::Fg(color::Black),
            message_line,
            style::Reset
        )
        .unwrap();
    }

    fn editor_draw_rows(&mut self) {
        for i in 0..self.window_size_row {
            let file_row = i + self.row_offset as u16;
            if file_row >= self.get_editor_line_length() {
                if self.get_editor_line_length() == 0 && i == (self.window_size_row / 3) {
                    let welcome_line = self.get_welcome_line();
                    write!(self.stdout, "{}", welcome_line).unwrap();
                } else {
                    let line = format!("~ ");
                    write!(self.stdout, "{}", line).unwrap();
                }
            } else {
                let mut len = self.editor_lines[file_row as usize].line.chars().count() as i16
                    - self.column_offset as i16;
                if len < 0 {
                    len = 0
                }

                if len > self.window_size_col as i16 {
                    len = self.window_size_col as i16
                }
                let whole_line = format!("{}", self.editor_lines[file_row as usize].line);

                let mut line = String::new();
                for (i, c) in whole_line.chars().enumerate() {
                    if i >= self.column_offset as usize
                        && i < (self.window_size_col + self.column_offset) as usize
                    {
                        line.push(c);
                    }
                }

                write!(self.stdout, "{}", line).unwrap();
            }

            if i < self.window_size_row {
                write!(self.stdout, "\r\n").unwrap();
            }
        }
    }

    fn editor_scroll(&mut self) {
        if self.cursor_y < self.get_editor_line_length() {
            self.render_x = self.editor_row_cx2rx();
        }

        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        }

        if self.cursor_y >= (self.row_offset + self.window_size_row) {
            self.row_offset = self.cursor_y - self.window_size_row + 1;
        }

        if self.render_x < self.column_offset {
            self.column_offset = self.render_x;
        }

        if self.render_x >= self.column_offset + self.window_size_col {
            self.column_offset = self.render_x - self.window_size_col + 1;
        }
    }

    pub fn run_viwer() {
        let mut viewer = Viewer::new();

        let file_name = "./hello_world.txt";
        viewer.set_status_message(String::from("HELP: Ctr-S = save | Ctr-C = quit"));

        viewer.editor_open(file_name);
        viewer.editor_refresh_screen();

        viewer.editor_process_key_press();

        write!(viewer.stdout, "{}", termion::cursor::Show).unwrap();
    }
}
