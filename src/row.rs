use crate::Highlight;
use std::cmp;

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
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            buf: vec![],
            render: vec![],
            highlight: vec![],
        }
    }
}
