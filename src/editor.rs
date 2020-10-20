use crate::Document;
use crate::Highlight;
use crate::Row;
use crate::Terminal;

use chrono::{DateTime, Duration, Utc};
use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use termion::event::Key;
use termion::*;

use std::fmt::{Display, Formatter, Result as FormatResult};

const KILO_VERSION: &str = "1.0";
const KILL_TAB_STOP: u8 = 4;
const QUIT_TIMES: u8 = 1; // 1 for dev.

const KEY_WORD_1: [&str; 15] = [
    "switch", "if", "while", "for", "break", "continue", "return", "else", "struct", "union",
    "typedef", "static", "enum", "class", "case",
];

const KEY_WORD_2: [&str; 8] = [
    "int", "long", "double", "float", "char", "unsigned", "signed", "void",
];

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

struct EditorSyntax {
    file_type: FileType,
    singleline_comment_start: String,
    multiline_comment_start: String,
    multiline_comment_end: String,
    highlight_number: bool,
    highlight_strings: bool,
}

enum FileType {
    C,
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            FileType::C => write!(f, "C"),
        }
    }
}

fn die(e: std::io::Error) {
    println!("{}", termion::clear::All);
    panic!(e)
}

#[derive(Default, Debug)]
pub struct Position {
    pub x: usize,
    pub render_x: usize,
    pub y: usize,
}

pub struct Editor {
    terminal: Terminal,
    position: Position,
    offset: Position,
    document: Document,
    file_name: Option<String>,
    editor_syntax: Option<EditorSyntax>,
    status_message: String,
    status_message_time: DateTime<Utc>,
    is_dirty: bool,
    quit_times: u8,
    should_quit: bool,
    increment_find: IncrementFind,
}

impl Editor {
    fn highlight_numbers(&self) -> bool {
        match &self.editor_syntax {
            Some(e_s) => return e_s.highlight_number,
            None => return false,
        }
    }

    fn highlight_strings(&self) -> bool {
        match &self.editor_syntax {
            Some(e_s) => return e_s.highlight_strings,
            None => return false,
        }
    }

    fn highlight_multi_comment(&self) -> bool {
        match &self.editor_syntax {
            Some(e_s) => return !e_s.multiline_comment_start.is_empty(),
            None => return false,
        }
    }

    fn new() -> Self {
        let row_offset = 0;
        let column_offset = 0;
        let status_message = String::new();
        let status_message_time = Utc::now();
        let is_dirty = false;
        let quit_times = QUIT_TIMES;
        let should_quit = false;

        Self {
            position: Position::default(),
            offset: Position::default(),
            terminal: Terminal::default(),
            document: Document::default(),
            file_name: None,
            editor_syntax: None,
            status_message,
            status_message_time,
            is_dirty,
            quit_times,
            should_quit,
            increment_find: IncrementFind::new(),
        }
    }

    fn editor_select_syntax_hilight(&mut self) {
        match &self.file_name {
            None => return,
            Some(name) => match Path::new(name).extension() {
                Some(s) => {
                    let extention = OsStr::to_str(s).unwrap_or("undefined");
                    if ["c", "h", "cpp"].contains(&extention) {
                        self.editor_syntax = Some(EditorSyntax {
                            file_type: FileType::C,
                            singleline_comment_start: String::from("//"),
                            multiline_comment_start: String::from("/*"),
                            multiline_comment_end: String::from("*/"),
                            highlight_number: true,
                            highlight_strings: true,
                        })
                    }
                }
                None => return,
            },
        }
    }

    fn saturated_add_x(&mut self) {
        if self.position.x < self.get_current_row_buf_length() {
            self.position.x = self.position.x + 1;
        } else {
            if self.position.y + 1 < self.document.len() {
                self.saturated_add_y();
                self.position.x = 0;
            }
        }
    }

    fn saturated_substract_x(&mut self) {
        if 0 < self.position.x {
            self.position.x = self.position.x - 1;
        } else {
            if 0 < self.position.y {
                self.saturated_substract_y();
                self.position.x = self.get_current_row_buf_length();
            }
        }
    }

    fn saturated_add_y(&mut self) {
        if self.position.y + 1 < self.document.len() {
            self.position.y = self.position.y + 1;
            if (self.position.x) > self.get_current_row_buf_length() {
                self.position.x = self.get_current_row_buf_length()
            }
        }
    }

    fn saturated_substract_y(&mut self) {
        if 0 < self.position.y {
            self.position.y = self.position.y - 1;
            if (self.position.x) > self.get_current_row_buf_length() {
                self.position.x = self.get_current_row_buf_length()
            }
        }
    }

    fn editor_row_insert_char(&mut self, c: char) {
        let mut new_buf = vec![];
        if self.position.x == self.get_current_row_buf_length() {
            new_buf = self
                .document
                .row(self.position.y as usize)
                .unwrap()
                .buf
                .clone();
            new_buf.push(c);
        } else {
            for (i, c_existed) in self
                .document
                .row(self.position.y as usize)
                .unwrap()
                .buf
                .iter()
                .enumerate()
            {
                if i == self.position.x as usize {
                    new_buf.push(c)
                }
                new_buf.push(c_existed.clone())
            }
        }
        self.document.replace_buf(self.position.y as usize, new_buf);
        self.saturated_add_x();
        self.is_dirty = true;
    }

    fn editor_insert_char(&mut self, c: char) {
        self.editor_row_insert_char(c)
    }

    fn editor_row_delete_character(&mut self) {
        let mut new_buf = vec![];
        for (i, c) in self
            .document
            .row(self.position.y as usize)
            .unwrap()
            .buf
            .iter()
            .enumerate()
        {
            if i == self.position.x as usize - 1 {
                continue;
            }
            new_buf.push(c.clone())
        }

        self.document.replace_buf(self.position.y as usize, new_buf);

        self.is_dirty = true
    }

    fn editor_delete_row(&mut self) {
        self.document.rows.remove(self.position.y as usize);
    }

    fn editor_row_append_string(&mut self, append_from_row_index: usize) {
        let append_to_row_index = append_from_row_index - 1;

        let mut move_to_buf = self.document.row(append_to_row_index).unwrap().buf.clone();
        let mut move_from_buf = self
            .document
            .row(append_from_row_index)
            .unwrap()
            .buf
            .clone();
        move_to_buf.append(&mut move_from_buf);

        self.document.replace_buf(append_to_row_index, move_to_buf);
        self.is_dirty = true;
    }

    fn editor_delete_char(&mut self) {
        if self.position.x == 0 && self.position.y == 0 {
            return;
        }

        if self.position.x > 0 {
            self.editor_row_delete_character();
            self.position.x = self.position.x - 1;
        } else {
            self.position.x = self
                .document
                .row(self.position.y as usize - 1)
                .unwrap()
                .buf
                .len();

            self.editor_row_append_string(self.position.y as usize);
            self.editor_delete_row();
            self.position.y = self.position.y - 1;
        }
    }

    fn split_line_resulted_from_enter_pressed(&self) -> (Vec<char>, Vec<char>) {
        let mut left_buf = vec![];
        let mut right_buf = vec![];

        for i in 0..self.get_current_row_buf_length() {
            let c = self.document.row(self.position.y as usize).unwrap().buf[i as usize];
            if i < self.position.x {
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
        if self.position.y == self.document.len() - 1 {
            for i in 0..self.document.len() - 1 {
                let el = &self.document.row(i as usize).unwrap();
                new_el_vec.push(Row {
                    buf: el.buf.clone(),
                    render: el.render.clone(),
                    highlight: vec![],
                })
            }
            new_el_vec.push(Row {
                buf: left_buf.clone(),
                render: vec![],
                highlight: vec![],
            });
            new_el_vec.push(Row {
                buf: right_buf.clone(),
                render: vec![],
                highlight: vec![],
            });
        } else {
            for (i, el) in self.document.rows.iter().enumerate() {
                // new line
                if i == self.position.y as usize + 1 {
                    new_el_vec.push(Row {
                        buf: right_buf.clone(),
                        render: vec![],
                        highlight: vec![],
                    })
                }

                // splited line
                if i == self.position.y as usize {
                    new_el_vec.push(Row {
                        buf: left_buf.clone(),
                        render: vec![],
                        highlight: vec![],
                    })
                // just  line
                } else {
                    new_el_vec.push(Row {
                        buf: el.buf.clone(),
                        render: vec![],
                        highlight: vec![],
                    })
                }
            }
        }
        self.document.rows = new_el_vec;
        self.saturated_add_y();
        self.position.x = 0
    }

    fn editor_prompt<F>(&mut self, prompt: String, mut incremental_callback: F) -> String
    where
        F: FnMut(&mut Self, &str, &Key, bool),
    {
        let mut input = String::new();
        self.set_status_message(format!("{}{}", prompt, input));
        self.editor_refresh_screen();

        loop {
            let r = Terminal::read_key();
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
        // TODO: editor_select_syntax_hilight after Save as ...
        match &self.file_name {
            None => return,
            Some(s) => match File::create(s) {
                Ok(mut f) => {
                    for i in 0..self.document.len() {
                        let buf = &self
                            .document
                            .row(i as usize)
                            .unwrap()
                            .buf
                            .iter()
                            .cloned()
                            .collect::<String>();
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

    fn editor_row_rx2cx(&mut self, render_x: usize) -> usize {
        let mut current_render_x = 0;
        let mut target_cursor_x = 0;

        let current_buf_row = &self.document.row(self.position.y as usize).unwrap().buf;
        for (_, c) in current_buf_row.iter().enumerate() {
            if *c == '\t' {
                current_render_x = current_render_x + (KILL_TAB_STOP as usize - 1)
                    - (current_render_x % KILL_TAB_STOP as usize)
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

        for _ in 0..self.document.len() {
            match self.increment_find.direction {
                IncrementFindDirection::Forward => current_row = current_row + 1,
                IncrementFindDirection::Backward => current_row = current_row - 1,
            }
            if current_row == -1 {
                current_row = self.document.len() as i16 - 1
            } else if current_row == self.document.len() as i16 {
                current_row = 0
            }

            let row = &self
                .document
                .row(current_row as usize)
                .unwrap()
                .render_string();

            if let Some(x) = row.find(&query) {
                self.increment_find.last_mached_row = Some(current_row);
                self.position.y = current_row as usize;
                self.position.x = self.editor_row_rx2cx(x);

                for _ in 0..query.chars().count() {
                    self.document.replace_char_highlight(
                        current_row as usize,
                        x + 1,
                        Highlight::Match,
                    );
                }
                break;
            }
        }
    }

    fn editor_find(&mut self) {
        let saved_cursor_x = self.position.x;
        let saved_cursor_y = self.position.y;
        let saved_column_offset = self.offset.y;
        let saved_row_offset = self.offset.x;

        let query = self.editor_prompt(String::from("Search:"), Self::on_incremental_find);
        if query.is_empty() {
            self.position.x = saved_cursor_x;
            self.position.y = saved_cursor_y;
            self.offset.y = saved_column_offset;
            self.offset.x = saved_row_offset;
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        dbg!(&pressed_key);
        match pressed_key {
            event::Key::Ctrl('c') | event::Key::Ctrl('q') => {
                //if self.is_dirty && self.quit_times > 0 {
                //    // TODO 他の作業をしたらquit_timesが回復するように
                //    self.set_status_message(format!(
                //    "WARNING!!! File has unsaved changes. Press Ctr-Q|C {} more times to quit",
                //    self.quit_times
                //));
                //    self.quit_times = self.quit_times - 1;
                //} else {
                self.should_quit = true;
                //}
            }
            event::Key::Backspace | event::Key::Ctrl('h') | event::Key::Delete => {
                self.editor_delete_char();
            }
            event::Key::Ctrl('s') => self.editor_save(),
            event::Key::Ctrl('f') => self.editor_find(),
            event::Key::Left | event::Key::Right | event::Key::Up | event::Key::Down => {
                self.move_cursor(pressed_key)
            }
            event::Key::Char(c) => {
                if c == '\n' {
                    self.editor_insert_new_line()
                } else {
                    self.editor_insert_char(c)
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn move_cursor(&mut self, key: Key) {
        match key {
            Key::Left => {
                self.saturated_substract_x();
            }
            Key::Right => {
                self.saturated_add_x();
            }
            Key::Up => {
                self.saturated_substract_y();
            }
            Key::Down => {
                self.saturated_add_y();
            }
            _ => {}
        }
    }

    fn editor_row_cx2rx(&mut self) -> usize {
        let mut render_x = 0;
        if self.position.x == 0 {
            return render_x;
        }
        let row = &self.document.row(self.position.y as usize).unwrap();
        for i in 0..self.position.x {
            if row.buf[i as usize] == '\t' {
                render_x =
                    render_x + (KILL_TAB_STOP as usize - 1) - (render_x % KILL_TAB_STOP as usize)
            }
            render_x = render_x + 1;
        }

        render_x
    }

    fn editor_refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.editor_scroll();
        Terminal::cursor_hide();
        Terminal::clear_screen();
        Terminal::cursor_position(&Position::default());

        self.editor_draw_rows();
        self.editor_draw_status_bar();
        self.editor_draw_message_bar();

        eprintln!(
            "cursor goto {}: {} (cursor_x: {}). row_offset: {}.  current_row_buf_length: {},current_row_render_length:{}, editor_line: {}",
            self.position.render_x,
            self.position.y,
            self.position.x,
            self.offset.x,
            self.get_current_row_buf_length(),
            self.get_current_row_render_length(),
            self.document.rows.len()
        );

        Terminal::cursor_position(&self.position);

        if self.should_quit {
            Terminal::clear_screen();
            println!("Good bye!!.\r")
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn get_welcome_line(&mut self) -> String {
        let mut welcom_message = format!("igc editor -- version {}", KILO_VERSION);
        let width = std::cmp::min(
            self.terminal.window_size_width as usize,
            welcom_message.len(),
        );

        let padding = (self.terminal.window_size_width as usize - width) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcom_message.truncate(width);
        format!("~{}{}", spaces, welcom_message)
    }

    fn get_editor_buffer_length(&self) -> u16 {
        let mut sum = 0;
        for line in &self.document.rows {
            sum = line.buf.len() * std::mem::size_of::<char>();
        }
        sum as u16
    }

    fn get_current_row_buf_length(&self) -> usize {
        self.document
            .row(self.position.y as usize)
            .unwrap()
            .buf
            .len()
    }

    fn get_current_row_render_length(&self) -> u16 {
        self.document
            .row(self.position.y as usize)
            .unwrap()
            .render
            .len() as u16
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
        let mut modified_status = "";
        if self.is_dirty {
            modified_status = "(modified)"
        }
        let mut status = format!(
            "{} - {} lines {}",
            display_file_name,
            self.document.len(),
            modified_status
        );
        let file_type = match &self.editor_syntax {
            Some(s) => format!("{}", &s.file_type),
            None => String::from("no ft"),
        };

        let mut right_status = format!(
            "{} | {}/{}",
            file_type,
            self.position.y + 1,
            self.document.len()
        );
        if status.chars().count() + right_status.chars().count()
            > self.terminal.window_size_width as usize
        {
            right_status = String::new();
        }
        for _ in status.chars().count()
            ..self.terminal.window_size_width as usize - right_status.chars().count()
        {
            status.push(' ')
        }
        let status_line = format!("{}{}", status, right_status);

        print!(
            "{}{}{}{}",
            color::Bg(color::LightMagenta),
            color::Fg(color::Black),
            status_line,
            style::Reset
        );
        print!("\r\n");
    }

    fn editor_draw_message_bar(&mut self) {
        let mut message_line = String::new();
        if self.status_message_time + Duration::seconds(5) < Utc::now() {
            return;
        }
        for (i, c) in self.status_message.chars().enumerate() {
            if i < self.terminal.window_size_width as usize {
                message_line.push(c)
            }
        }

        print!(
            "{}{}{}{}",
            color::Bg(color::LightMagenta),
            color::Fg(color::Black),
            message_line,
            style::Reset
        )
    }

    fn editor_update_row(&mut self) {
        let mut new_vec = vec![];
        for (_, line) in self.document.rows.iter().enumerate() {
            let mut render = vec![]; // TODO: pushではなくて、メモリ確保して処理する
            for c in line.buf.iter() {
                if *c == '\t' {
                    for _ in 0..KILL_TAB_STOP {
                        render.push(' ');
                    }
                } else {
                    render.push(c.clone());
                }
            }
            new_vec.push(render);
        }
        for i in 0..self.document.len() {
            self.document
                .replace_render(i as usize, new_vec[i as usize].clone());
        }

        self.editor_update_syntax()
    }

    fn draw_row(&self, row: &Row) {
        for (j, c) in row.render.iter().enumerate() {
            let mut current_color: Highlight = Highlight::Normal;
            if j >= self.offset.y as usize
                && j < (self.terminal.window_size_width + self.offset.y as u16) as usize
            {
                let peek_color = row.highlight[j];
                if current_color != peek_color {
                    current_color = peek_color;
                    let ansi_value = current_color.editor_syntax_to_color();
                    print!("{}{}{}", style::Reset, color::Fg(ansi_value), c)
                } else {
                    let ansi_value = current_color.editor_syntax_to_color();
                    print!("{}{}", color::Fg(ansi_value), c);
                }
            }
        }
    }

    fn editor_draw_rows(&mut self) {
        self.editor_update_row();
        for i in 0..self.terminal.window_size_height {
            let file_row = i as usize + self.offset.x;
            if file_row >= self.document.len() {
                if self.document.is_empty() && i == (self.terminal.window_size_height / 3) {
                    let welcome_line = self.get_welcome_line();
                    print!("{}", welcome_line);
                } else {
                    let line = format!("~ ");
                    print!("{}", line);
                }
            } else {
                if let Some(row) = self.document.row(file_row as usize) {
                    self.draw_row(row)
                }
            }

            if i < self.terminal.window_size_height {
                print!("\r\n{}", style::Reset);
            }
        }
    }

    fn editor_scroll(&mut self) {
        if self.position.y < self.document.len() as usize {
            self.position.render_x = self.editor_row_cx2rx();
        }

        if self.position.y < self.offset.x {
            self.offset.x = self.position.y;
        }

        if self.position.y >= (self.offset.x + self.terminal.window_size_height as usize) {
            self.offset.x = self.position.y - self.terminal.window_size_height as usize + 1;
        }

        if self.position.render_x < self.offset.y {
            self.offset.y = self.position.render_x;
        }

        if self.position.render_x >= self.offset.y + self.terminal.window_size_width as usize {
            self.offset.y = self.position.render_x - self.terminal.window_size_width as usize + 1;
        }
    }

    fn is_digit(c: &char) -> bool {
        match *c {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => return true,
            _ => return false,
        }
    }

    fn default_hilight(&mut self) {
        let mut highlight = vec![];
        for e_l in &self.document.rows {
            let mut line = vec![];
            for _c in &e_l.render {
                line.push(Highlight::Normal);
            }
            highlight.push(line);
        }
        for i in 0..self.document.len() {
            self.document
                .replace_highlight(i as usize, highlight[i as usize].clone());
        }
    }

    fn single_comment_start_length(&self) -> usize {
        match &self.editor_syntax {
            Some(e_s) => return e_s.singleline_comment_start.chars().count(),
            None => 0,
        }
    }

    fn has_singleline_comment_started(&self, index: usize, render_row: &Vec<char>) -> bool {
        let single_comment_start = match &self.editor_syntax {
            Some(e_s) => e_s.singleline_comment_start.clone(),
            None => String::from(""),
        };
        let mut s = String::new();
        for (i, c) in render_row.iter().enumerate() {
            if i >= index && i < index + self.single_comment_start_length() {
                s.push(*c)
            }
        }

        single_comment_start == s
    }

    fn is_separator(c: &char) -> bool {
        match c {
            ' ' => true,  // space
            '\0' => true, // null,
            ',' | '.' | '(' | ')' | '+' | '-' | '/' | '*' | '=' | '~' | '%' | '<' | '>' | '['
            | ']' | ';' => true, // separator chars
            _ => false,
        }
    }

    fn get_word(&self, row: &str, start_index: usize) -> String {
        let mut s = String::new();
        for (i, c) in row.chars().enumerate() {
            if i < start_index {
                continue;
            }

            if Editor::is_separator(&c) {
                break;
            };

            s.push(c);
        }

        s
    }

    fn multi_comment_start(&self) -> String {
        match &self.editor_syntax {
            Some(e_s) => return e_s.multiline_comment_start.clone(),
            None => return String::new(),
        }
    }
    fn multi_comment_end(&self) -> String {
        match &self.editor_syntax {
            Some(e_s) => return e_s.multiline_comment_end.clone(),
            None => return String::new(),
        }
    }

    fn multi_comment_end_len(&self) -> u16 {
        match &self.editor_syntax {
            Some(e_s) => return e_s.multiline_comment_end.chars().count() as u16,
            None => return 0,
        }
    }

    fn multi_comment_start_len(&self) -> u16 {
        match &self.editor_syntax {
            Some(e_s) => return e_s.multiline_comment_start.chars().count() as u16,
            None => return 0,
        }
    }

    fn str_compare(&self, row: &Vec<char>, start_index: usize, keyword: &String) -> bool {
        let mut s = String::new();
        for (i, c) in row.iter().enumerate() {
            if i >= start_index && i < start_index + keyword.chars().count() {
                s.push(*c)
            }
        }
        keyword == &s
    }

    fn editor_update_syntax(&mut self) {
        if self.editor_syntax.is_none() {
            self.default_hilight();
            return;
        }
        let previous_separator = true;
        let mut highlight_matrix = vec![];
        let mut is_in_string: bool = false;
        let mut in_string: char = '\0';
        let mut is_in_comment: bool = false;

        for (column_index, e_l) in self.document.rows.iter().enumerate() {
            let mut highlight = vec![Highlight::Normal; e_l.render.len()];
            let row = &e_l.render;
            let mut row_index = 0;
            let mut preivious_highlight = Highlight::Normal;

            while row_index < e_l.render.len() {
                let c = &row[row_index];

                if row_index > 0 {
                    preivious_highlight = highlight[row_index - 1];
                }

                // higlight single comment
                if !is_in_string && !is_in_comment {
                    if self.has_singleline_comment_started(row_index, &e_l.render) {
                        while row_index < e_l.render.len() {
                            highlight[row_index] = Highlight::Comment;
                            row_index = row_index + 1
                        }
                        break;
                    }
                }

                // higlight mult comment
                if self.highlight_multi_comment() && !is_in_string {
                    if is_in_comment {
                        highlight[row_index] = Highlight::MultiComment;

                        if self.str_compare(&row, row_index, &self.multi_comment_end()) {
                            is_in_comment = false;
                            for _ in 0..self.multi_comment_end_len() {
                                highlight[row_index] = Highlight::MultiComment;
                                row_index = row_index + 1;
                            }
                            continue;
                        } else {
                            row_index = row_index + 1;
                            continue;
                        }
                    } else if self.str_compare(&row, row_index, &self.multi_comment_start()) {
                        for _ in 0..self.multi_comment_start_len() {
                            highlight[row_index] = Highlight::MultiComment;
                            row_index = row_index + 1;
                        }
                        is_in_comment = true;
                        continue;
                    }
                }

                // higlight number
                if self.highlight_numbers() {
                    if Self::is_digit(c) || (*c == '.' && preivious_highlight == Highlight::Number)
                    {
                        highlight[row_index] = Highlight::Number;
                        row_index = row_index + 1;
                        continue;
                    }
                }

                // highlight strings
                if self.highlight_strings() {
                    if is_in_string {
                        highlight[row_index] = Highlight::String;

                        if row_index > 0
                            && e_l.render[row_index as usize - 1] == '\\'
                            && row_index < self.get_current_row_render_length() as usize
                        {
                            row_index = row_index + 1;
                            continue;
                        }

                        if *c == in_string {
                            is_in_string = false;
                        }
                        row_index = row_index + 1;
                        continue;
                    } else {
                        if *c == '"' || *c == '\'' {
                            is_in_string = true;
                            in_string = *c;
                            highlight[row_index] = Highlight::String;

                            row_index = row_index + 1;
                            continue;
                        }
                    }
                }
                // hilight keywords
                if previous_separator {
                    let row = &self.document.row(column_index).unwrap().render_string();
                    let word = &*self.get_word(row, row_index);
                    if KEY_WORD_1.contains(&word) {
                        for _ in 0..word.chars().count() {
                            highlight[row_index] = Highlight::Keyword1;
                            row_index = row_index + 1;
                        }
                        continue;
                    }

                    if KEY_WORD_2.contains(&word) {
                        for _ in 0..word.chars().count() {
                            highlight[row_index] = Highlight::Keyword2;
                            row_index = row_index + 1;
                        }
                        continue;
                    }
                }
                row_index = row_index + 1;
            }

            highlight_matrix.push(highlight);
        }
        for i in 0..self.document.len() {
            self.document
                .replace_highlight(i as usize, highlight_matrix[i as usize].clone());
        }
    }

    pub fn default() -> Self {
        let mut editor = Editor::new();

        editor.set_status_message(String::from(
            "HELP: Ctr-S = save | Ctr-C = quit | Ctrl-F = find",
        ));

        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let document = Document::open(file_name).unwrap_or_default();

            editor.file_name = Some(String::from(file_name));
            editor.editor_select_syntax_hilight();
            document
        } else {
            Document::default()
        };
        editor.document = document;

        editor
    }

    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.editor_refresh_screen() {
                die(error)
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error)
            }
        }
    }
}
