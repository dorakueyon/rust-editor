//#![warn(clippy::all, clippy::pedantic)]

use editor::Editor;

mod document;
mod editor;
mod highlight;
mod row;
mod terminal;

pub use document::Document;
pub use highlight::Highlight;
pub use row::Row;
pub use terminal::Terminal;
pub use editor::Position;

fn main() {
    Editor::default().run()
}
