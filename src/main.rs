use editor::Editor;

mod editor;
mod document;
mod highlight;
mod row;

pub use document::Document;
pub use highlight::Highlight;
pub use row::Row;

fn main() {
    Editor::run()
}
