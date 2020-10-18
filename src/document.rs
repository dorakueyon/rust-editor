use crate::Row;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Default)]
pub struct Document{
    pub rows: Vec<Row>,
}
impl Document {
    pub fn open(file_name: &str) -> Document {

let file = match File::open(file_name) {
            Err(why) => panic!("couldn't open {}: {}", file_name, why),
            Ok(file) => file,
        };

        let mut editor_lines = vec![];
        for line in BufReader::new(file).lines() {
            match line {
                Ok(s) => {
                    let mut buf = vec![];
                    for c in s.trim_end().chars() {
                        buf.push(c);
                    }
                    editor_lines.push(Row{
                        buf,
                        render: vec![],
                        highlight: vec![],
                    });
                }
                Err(_) => {}
            }
        }
      Document {
        rows: editor_lines,
      }
    }
}

