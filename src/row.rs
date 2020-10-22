use crate::Highlight;
use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct Row {
    pub buf: Vec<char>,
    pub render: Vec<char>,
    pub highlight: Vec<Highlight>,
}

impl Row {
    pub fn render_string(&self) -> String {
        let mut line = String::new();
        for c in &self.render {
            line.push(c.clone());
        }
        line
    }

    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.render.len());
        let start = cmp::min(start, end);
        self.render[start..end].iter().collect()
    }

    pub fn buf_len(&self) -> usize {
        self.buf.len()
    }
}

impl From<&String> for Row {
    fn from(slice: &String) -> Self {
        let mut buf = vec![];
        for c in slice.trim_end().chars() {
            buf.push(c);
        }
        //for grapheme in slice.graphemes(true) {
        //    dbg!(grapheme);
        //    buf.push(grapheme.chars().next().unwrap_or(' '));
        //}
        Self {
            buf,
            render: vec![],
            highlight: vec![],
        }
    }
}
