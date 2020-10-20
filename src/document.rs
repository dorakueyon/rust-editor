use crate::Highlight;
use crate::Row;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Default)]
pub struct Document {
    pub rows: Vec<Row>,
}
impl Document {
    pub fn open(file_name: &str) -> Result<Document, std::io::Error> {
        let file = File::open(file_name)?;

        let mut editor_lines = vec![];
        for line in BufReader::new(file).lines() {
            match line {
                Ok(s) => editor_lines.push(Row::from(&s)),
                Err(_) => {}
            }
        }
        Ok(Document { rows: editor_lines })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn replace_buf(&mut self, index: usize, new_buf: Vec<char>) -> Result<(), std::io::Error> {
        self.rows[index].buf = new_buf;
        Ok(())
    }

    pub fn replace_render(
        &mut self,
        index: usize,
        new_render: Vec<char>,
    ) -> Result<(), std::io::Error> {
        self.rows[index].render = new_render;
        Ok(())
    }

    pub fn replace_highlight(
        &mut self,
        index: usize,
        new_highlight: Vec<Highlight>,
    ) -> Result<(), std::io::Error> {
        self.rows[index].highlight = new_highlight;
        Ok(())
    }

    pub fn replace_char_highlight(
        &mut self,
        row_index: usize,
        char_index: usize,
        new_highlight: Highlight,
    ) -> Result<(), std::io::Error> {
        self.rows[row_index].highlight[char_index] = new_highlight;
        Ok(())
    }
}
