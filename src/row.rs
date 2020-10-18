
use crate::Highlight;
#[derive(Debug, Clone)]
pub struct Row{
    pub buf: Vec<char>,
    pub render: Vec<char>,
    pub highlight: Vec<Highlight>,
}

impl  Row{
    pub fn render_string(&self) -> String {
        let mut line = String::new();
        for c in &self.render {
            line.push(c.clone());
        }
        line
    }
}




