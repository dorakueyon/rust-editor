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

use termion::screen::AlternateScreen;

const KILO_VERSION: &str = "1.0";
const KILL_TAB_STOP: u8 = 4;
const STATUS_LINE_LENGTH: u16 = 2;
const QUIT_TIMES: u8 = 1; // 1 for dev.

#[derive(Debug)]
pub enum IncrementFindDirection {
    Forward,
    Backward,
}

#[derive(Debug)]
pub struct IncrementFind {
    last_mached_row: Option<i16>,
    direction: IncrementFindDirection,
}

impl IncrementFind {
    fn new() -> Self {
        Self {
            last_mached_row: None,
            direction: IncrementFindDirection::Forward,
        }
    }
}

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
    is_dirty: bool,
    quit_times: u8,
    increment_find: IncrementFind,
}

enum EditorHighlight {
    Normal,
    Number,
}

#[derive(Debug)]
struct EditorLine {
    buf: String,
    render: String,
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
        let is_dirty = false;
        let quit_times = QUIT_TIMES;

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
            is_dirty,
            quit_times,
            increment_find: IncrementFind::new(),
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
                Ok(s) => {
                    let mut buf = String::new();
                    for c in s.trim_end().chars() {
                        buf.push(c);
                    }
                    editor_lines.push(EditorLine {
                        buf,
                        render: String::new(),
                    });
                }
                Err(_) => {}
            }
        }
        self.editor_lines = editor_lines;

        self.file_name = Some(String::from(file_name));
    }

    fn saturated_add_x(&mut self) {
        if self.cursor_x < self.get_current_row_buf_length() {
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
                self.cursor_x = self.get_current_row_buf_length();
            }
        }
    }

    fn saturated_add_y(&mut self) {
        if self.cursor_y + 1 < self.get_editor_line_length() {
            self.cursor_y = self.cursor_y + 1;
            if (self.cursor_x) > self.get_current_row_buf_length() {
                self.cursor_x = self.get_current_row_buf_length()
            }
        }
    }

    fn saturated_substract_y(&mut self) {
        if 0 < self.cursor_y {
            self.cursor_y = self.cursor_y - 1;
            if (self.cursor_x) > self.get_current_row_buf_length() {
                self.cursor_x = self.get_current_row_buf_length()
            }
        }
    }

    fn editor_row_insert_char(&mut self, c: char) {
        let mut new_buf = String::new();
        if self.cursor_x == self.get_current_row_buf_length() {
            new_buf = format!("{}{}", self.editor_lines[self.cursor_y as usize].buf, c)
        } else {
            for (i, c_existed) in self.editor_lines[self.cursor_y as usize]
                .buf
                .chars()
                .enumerate()
            {
                if i == self.cursor_x as usize {
                    new_buf.push(c)
                }
                new_buf.push(c_existed)
            }
        }
        self.editor_lines[self.cursor_y as usize].buf = new_buf;
        self.saturated_add_x();
        self.is_dirty = true;
    }

    fn editor_insert_char(&mut self, c: char) {
        self.editor_row_insert_char(c)
    }

    fn editor_row_delete_character(&mut self) {
        let mut new_buf = String::new();
        for (i, c) in self.editor_lines[self.cursor_y as usize]
            .buf
            .chars()
            .enumerate()
        {
            if i == self.cursor_x as usize - 1 {
                continue;
            }
            new_buf.push(c)
        }

        self.editor_lines[self.cursor_y as usize].buf = new_buf;

        self.is_dirty = true
    }

    fn editor_delete_row(&mut self) {
        self.editor_lines.remove(self.cursor_y as usize);
    }

    fn editor_row_append_string(&mut self, append_from_row_index: usize) {
        let append_to_row_index = append_from_row_index - 1;

        let copy_to_buf = self.editor_lines[append_to_row_index].buf.clone();
        let copy_from_buf = self.editor_lines[append_from_row_index].buf.clone();

        let new_buf = format!("{}{}", copy_to_buf, copy_from_buf);

        self.editor_lines[append_to_row_index].buf = new_buf;
        self.is_dirty = true;
    }

    fn editor_delete_char(&mut self) {
        if self.cursor_x == 0 && self.cursor_y == 0 {
            return;
        }

        if self.cursor_x > 0 {
            self.editor_row_delete_character();
            self.cursor_x = self.cursor_x - 1;
        } else {
            self.cursor_x = (self.editor_lines[self.cursor_y as usize - 1]
                .buf
                .chars()
                .count()) as u16;

            self.editor_row_append_string(self.cursor_y as usize);
            self.editor_delete_row();
            self.cursor_y = self.cursor_y - 1;
        }
    }

    fn split_line_resulted_from_enter_pressed(&self) -> (String, String) {
        let mut left_buf = String::new();
        let mut right_buf = String::new();

        for i in 0..self.get_current_row_buf_length() {
            let c = self.editor_lines[self.cursor_y as usize]
                .buf
                .chars()
                .nth(i as usize)
                .unwrap();
            if i < self.cursor_x {
                left_buf.push(c)
            } else {
                right_buf.push(c)
            }
        }
        (left_buf, right_buf)
    }

    fn editor_insert_new_line(&mut self) {
        let mut new_el_vec = vec![];
        let (left_buf, right_buf) = self.split_line_resulted_from_enter_pressed();
        if self.cursor_y == self.get_editor_line_length() - 1 {
            for i in 0..self.get_editor_line_length() - 1 {
                let el = &self.editor_lines[i as usize];
                new_el_vec.push(EditorLine {
                    buf: el.buf.clone(),
                    render: el.render.clone(),
                })
            }
            new_el_vec.push(EditorLine {
                buf: left_buf.clone(),
                render: String::new(),
            });
            new_el_vec.push(EditorLine {
                buf: right_buf.clone(),
                render: String::new(),
            });
        } else {
            for (i, el) in self.editor_lines.iter().enumerate() {
                // new line
                if i == self.cursor_y as usize + 1 {
                    new_el_vec.push(EditorLine {
                        buf: right_buf.clone(),
                        render: String::new(),
                    })
                }

                // splited line
                if i == self.cursor_y as usize {
                    new_el_vec.push(EditorLine {
                        buf: left_buf.clone(),
                        render: String::new(),
                    })
                // just  line
                } else {
                    new_el_vec.push(EditorLine {
                        buf: el.buf.clone(),
                        render: el.render.clone(),
                    })
                }
            }
        }
        self.editor_lines = new_el_vec;
        self.saturated_add_y();
        self.cursor_x = 0
    }

    fn editor_prompt<F>(&mut self, prompt: String, mut incremental_callback: F) -> String
    where
        F: FnMut(&mut Self, &str, &Key, bool),
    {
        let mut input = String::new();
        self.set_status_message(format!("{}{}", prompt, input));
        self.editor_refresh_screen();

        for r in stdin().keys() {
            match &r {
                Ok(Key::Esc) => {
                    self.set_status_message(String::new());
                    incremental_callback(self, &input, &r.unwrap(), true);
                    return String::new();
                }
                Ok(Key::Backspace) | Ok(Key::Delete) => {
                    input.pop();
                }
                Ok(Key::Right) | Ok(Key::Left) | Ok(Key::Down) | Ok(Key::Up)
                | Ok(Key::Char('\n')) => match &r {
                    Ok(key) => incremental_callback(self, &input, &key, false),
                    Err(_) => {}
                },
                Ok(event::Key::Char(c)) => {
                    eprintln!("char!!!{}", &c);
                    input.push(c.clone());
                    incremental_callback(self, &input, &r.unwrap(), false);
                }
                _ => {}
            }
            self.set_status_message(format!("{}{}", prompt, input));
            self.editor_refresh_screen();
        }

        prompt
    }

    fn editor_save(&mut self) {
        // TODO: Save as ...
        match &self.file_name {
            None => return,
            Some(s) => match File::create(s) {
                Ok(mut f) => {
                    for i in 0..self.get_editor_line_length() {
                        let buf = &self.editor_lines[i as usize].buf;
                        f.write_all(buf.as_bytes()).unwrap();
                        f.write_all(b"\r\n").unwrap();
                    }
                    self.is_dirty = false;
                    self.set_status_message(format!(
                        "{} bytes written to disk",
                        self.get_editor_buffer_length()
                    ));
                }

                Err(e) => self.set_status_message(format!("Can't save! I/O error: {}", e)),
            },
        }
    }

    fn editor_row_rx2cx(&mut self, render_x: u16) -> u16 {
        let mut current_render_x: u16 = 0;
        let mut target_cursor_x: u16 = 0;

        let current_buf_row = &self.editor_lines[self.cursor_y as usize].buf;
        for (_, c) in current_buf_row.chars().enumerate() {
            if c == '\t' {
                current_render_x = current_render_x + (KILL_TAB_STOP as u16 - 1)
                    - (current_render_x % KILL_TAB_STOP as u16)
            };
            current_render_x = current_render_x + 1;
            target_cursor_x = target_cursor_x + 1;

            if current_render_x >= render_x {
                return target_cursor_x;
            }
        }
        return target_cursor_x;
    }

    fn on_incremental_find(&mut self, query: &str, key: &Key, end: bool) {
        if end {
            self.increment_find = IncrementFind::new()
        }
        match key {
            Key::Right | Key::Down | Key::Char('\n') => {
                self.increment_find.direction = IncrementFindDirection::Forward
            }
            Key::Left | Key::Up => self.increment_find.direction = IncrementFindDirection::Backward,
            _ => {
                self.increment_find = IncrementFind::new();
            }
        };

        let mut current_row = -1;
        if let Some(i) = self.increment_find.last_mached_row {
            current_row = i
        }

        for _ in 0..self.get_editor_line_length() {
            match self.increment_find.direction {
                IncrementFindDirection::Forward => current_row = current_row + 1,
                IncrementFindDirection::Backward => current_row = current_row - 1,
            }
            if current_row == -1 {
                current_row = self.get_editor_line_length() as i16 - 1
            } else if current_row == self.get_editor_line_length() as i16 {
                current_row = 0
            }

            let row = &self.editor_lines[current_row as usize].render;

            if let Some(x) = row.find(&query) {
                self.increment_find.last_mached_row = Some(current_row as i16);
                self.cursor_y = current_row as u16;
                self.cursor_x = self.editor_row_rx2cx(x as u16);
                break;
            }
        }
    }

    fn editor_find(&mut self) {
        let saved_cursor_x = self.cursor_x;
        let saved_cursor_y = self.cursor_y;
        let saved_column_offset = self.column_offset;
        let saved_row_offset = self.row_offset;

        let query = self.editor_prompt(String::from("Search:"), Self::on_incremental_find);
        if query.is_empty() {
            self.cursor_x = saved_cursor_x;
            self.cursor_y = saved_cursor_y;
            self.column_offset = saved_column_offset;
            self.row_offset = saved_row_offset;
        }
    }

    fn editor_process_key_press(&mut self) {
        for c in stdin().keys() {
            //dbg!(&c);
            dbg!(&self.editor_lines[self.cursor_y as usize]);
            match c {
                Ok(event::Key::Ctrl('c')) | Ok(event::Key::Ctrl('q')) => {
                    if self.is_dirty && self.quit_times > 0 {
                        // TODO 他の作業をしたらquit_timesが回復するように
                        self.set_status_message(format!(
                        "WARNING!!! File has unsaved changes. Press Ctr-Q|C {} more times to quit",
                        self.quit_times
                    ));
                        self.quit_times = self.quit_times - 1;
                    } else {
                        break;
                    }
                }
                Ok(event::Key::Backspace) | Ok(event::Key::Ctrl('h')) | Ok(event::Key::Delete) => {
                    self.editor_delete_char();
                }
                Ok(event::Key::Ctrl('s')) => self.editor_save(),
                Ok(event::Key::Ctrl('f')) => self.editor_find(),
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
                        self.editor_insert_new_line()
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
        let mut render_x = 0;
        if self.cursor_x == 0 {
            return render_x;
        }
        let row = &self.editor_lines[self.cursor_y as usize];
        for i in 0..self.cursor_x {
            dbg!(&self.cursor_x);
            if row.buf.chars().nth(i as usize).unwrap() as char == '\t' {
                render_x = render_x + (KILL_TAB_STOP as u16 - 1) - (render_x % KILL_TAB_STOP as u16)
            }
            render_x = render_x + 1;
        }

        render_x
    }

    fn editor_refresh_screen(&mut self) {
        self.editor_scroll();
        write!(self.stdout, "{}", clear::All).unwrap();
        write!(self.stdout, "{}", cursor::Goto(1, 1)).unwrap();

        self.editor_draw_rows();
        self.editor_draw_status_bar();
        self.editor_draw_message_bar();

        eprintln!(
            "cursor goto {}: {} (cursor_x: {}). row_offset: {}.  current_row_buf_length: {},current_row_render_length:{}, editor_line: {}",
            self.render_x,
            self.cursor_y,
            self.cursor_x,
            self.row_offset,
            self.get_current_row_buf_length(),
            self.get_current_row_render_length(),
            self.editor_lines.len()
        );
        write!(
            self.stdout,
            "{}{}",
            cursor::BlinkingBar,
            cursor::Goto(
                self.render_x + 1 - self.column_offset,
                self.cursor_y + 1 - self.row_offset
            ),
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
            sum = sum + line.buf.as_bytes().len()
        }
        sum as u16
    }

    fn get_current_row_buf_length(&self) -> u16 {
        self.editor_lines[self.cursor_y as usize]
            .buf
            .chars()
            .count() as u16
    }

    fn get_current_row_render_length(&self) -> u16 {
        self.editor_lines[self.cursor_y as usize]
            .render
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
            }
            None => display_file_name = String::from("[No Name]"),
        }
        let mut status = format!(
            "{} - {} lines {}",
            display_file_name,
            self.get_editor_line_length(),
            if self.is_dirty { "(modified)" } else { "" }
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

    fn editor_update_row(&mut self) {
        let mut new_vec = vec![];
        for (_, line) in self.editor_lines.iter().enumerate() {
            let mut render = String::new(); // TODO: pushではなくて、メモリ確保して処理する
            for c in line.buf.chars() {
                if c == '\t' {
                    for _ in 0..KILL_TAB_STOP {
                        render.push(' ');
                    }
                } else {
                    render.push(c);
                }
            }
            new_vec.push(render);
        }
        for i in 0..self.get_editor_line_length() {
            self.editor_lines[i as usize].render = new_vec[i as usize].clone();
        }
    }

    fn editor_draw_rows(&mut self) {
        self.editor_update_row();
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
                let mut len = self.editor_lines[file_row as usize].render.chars().count() as i16
                    - self.column_offset as i16;
                if len < 0 {
                    len = 0
                }

                if len > self.window_size_col as i16 {
                    len = self.window_size_col as i16
                }
                let whole_render_line = format!("{}", self.editor_lines[file_row as usize].render);

                let mut line = String::new();
                for (i, c) in whole_render_line.chars().enumerate() {
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

    fn editor_update_syntax() {}

    pub fn run_viwer() {
        let mut viewer = Viewer::new();

        let file_name = "./hello_world.txt";
        viewer.set_status_message(String::from(
            "HELP: Ctr-S = save | Ctr-C = quit | Ctrl-F = find",
        ));

        viewer.editor_open(file_name);
        viewer.editor_refresh_screen();

        viewer.editor_process_key_press();

        write!(viewer.stdout, "{}", termion::cursor::Show).unwrap();
    }
}
