#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::must_use_candidate,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else,
    clippy::as_conversions
)]
mod document;
mod editor;
mod highlighting;
mod row;
mod terminal;

pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use editor::SearchDirection;
pub use row::Row;
pub use terminal::Terminal;

fn main() {
    Editor::default().run();
}
