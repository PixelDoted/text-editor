use std::fmt::Display;

pub mod action;
pub mod cursor;
pub mod view;

pub enum ViewMode {
    Normal,
    Insert,
    Select,
}

impl Display for ViewMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ViewMode::Normal => "NORMAL",
            ViewMode::Insert => "INSERT",
            ViewMode::Select => "SELECT",
        })
    }
}
