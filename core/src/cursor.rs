#[derive(Default)]
pub struct Cursor {
    /// Line Index
    pub row: usize,
    /// Char on Line Index
    pub col: usize,
    pub ghost_col: usize,
}

impl Cursor {
    pub fn set_row(&mut self, row: usize) {
        self.row = row;
    }

    pub fn set_column(&mut self, col: usize) {
        self.col = col;
        self.ghost_col = col;
    }
}

pub enum MoveCursor {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
}
