use ropey::Rope;

use crate::cursor::{Cursor, MoveCursor};

pub trait View {
    /// Inserts [`text`] into the buffer, shifting the cursor by `text.len()`
    fn insert(&mut self, text: &str) -> Result<(), ropey::Error>; // TODO: use a custom error type

    /// Inserts a char into the buffer, shifting the cursor by `1`
    fn insert_char(&mut self, c: char) -> Result<(), ropey::Error>; // TODO: use a custom error type

    /// Removes a char at the cursor.
    /// [`prev`] is used for `Backspace`
    fn remove(&mut self, prev: bool) -> Result<(), ropey::Error>; // TODO: use a custom error type

    /// A reference to the cursor
    fn cursor(&self) -> &Cursor;
}

pub struct TextView {
    pub buffer: Rope,

    cursor: Cursor,
    selection: Option<Cursor>,
}

impl TextView {
    pub fn new(buffer: Rope) -> Self {
        Self {
            buffer,

            cursor: Cursor::default(),
            selection: None,
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self::new(Rope::from_str(text))
    }
}

impl View for TextView {
    fn insert(&mut self, text: &str) -> Result<(), ropey::Error> {
        let line_start = self.buffer.line_to_char(self.cursor.row);
        let result = self.buffer.try_insert(line_start + self.cursor.col, text);

        self.move_cursor(MoveCursor::Right(text.len()));
        result
    }

    fn insert_char(&mut self, c: char) -> Result<(), ropey::Error> {
        let line_start = self.buffer.line_to_char(self.cursor.row);
        let result = self.buffer.try_insert_char(line_start + self.cursor.col, c);

        if c == '\n' {
            self.cursor.set_column(0);
            self.cursor.row += 1;
        } else {
            self.move_cursor(MoveCursor::Right(1));
        }
        result
    }

    fn remove(&mut self, prev: bool) -> Result<(), ropey::Error> {
        if prev {
            if self.cursor.row == 0 && self.cursor.col == 0 {
                return Ok(());
            }

            let line_start = self.buffer.line_to_char(self.cursor.row);
            let idx = line_start + self.cursor.col - 1;
            self.move_cursor(MoveCursor::Left(1));

            if let Some(selection_cursor) = self.selection.as_ref() {
                let line_start = self.buffer.line_to_char(selection_cursor.row);
                let end_idx = line_start + selection_cursor.col - 1;
                self.buffer.try_remove(idx..=end_idx)
            } else {
                self.buffer.try_remove(idx..=idx)
            }
        } else {
            if self.cursor.col >= self.buffer.line(self.cursor.row).len_chars() {
                return Ok(());
            }

            let line_start = self.buffer.line_to_char(self.cursor.row);
            let idx = line_start + self.cursor.col;

            self.buffer.try_remove(idx..=idx)
        }
    }

    fn cursor(&self) -> &Cursor {
        &self.cursor
    }
}

impl TextView {
    pub fn line_end_index(&self, row: usize) -> usize {
        let line = self.buffer.line(row);
        let mut idx = line.len_chars();
        if idx != 0 && line.char(idx - 1) == '\n' {
            idx -= 1;
        }

        idx
    }

    pub fn move_cursor(&mut self, mov: MoveCursor) {
        match mov {
            MoveCursor::Up(delta) => {
                self.cursor.row = self.cursor.row.saturating_sub(delta);
                self.cursor.col = self
                    .cursor
                    .ghost_col
                    .min(self.line_end_index(self.cursor.row));
            }
            MoveCursor::Down(delta) => {
                self.cursor.row += delta;
                if self.cursor.row >= self.buffer.len_lines() {
                    self.cursor.row = self.buffer.len_lines() - 1;
                }

                self.cursor.col = self
                    .cursor
                    .ghost_col
                    .min(self.line_end_index(self.cursor.row));
            }
            MoveCursor::Left(delta) => {
                if self.cursor.col != self.cursor.ghost_col {
                    self.cursor.ghost_col = self.cursor.col;
                }

                for _ in 0..delta {
                    if self.cursor.ghost_col == 0 {
                        if self.cursor.row > 0 {
                            self.cursor.row -= 1;
                            self.cursor.ghost_col = self.line_end_index(self.cursor.row);
                        } else {
                            break;
                        }
                    } else {
                        self.cursor.ghost_col -= 1;
                    }
                }

                self.cursor.col = self.cursor.ghost_col;
            }
            MoveCursor::Right(delta) => {
                if self.cursor.col != self.cursor.ghost_col {
                    self.cursor.ghost_col = self.cursor.col;
                }

                for _ in 0..delta {
                    if self.cursor.ghost_col == self.line_end_index(self.cursor.row) {
                        let lines = self.buffer.len_lines();
                        if lines > 0 && self.cursor.row < lines - 1 {
                            self.cursor.row += 1;
                            self.cursor.ghost_col = 0;
                        } else {
                            break;
                        }
                    } else {
                        self.cursor.ghost_col += 1;
                    }
                }

                self.cursor.col = self.cursor.ghost_col;
            }
            MoveCursor::Home => {
                self.cursor.set_column(0);
            }
            MoveCursor::End => {
                self.cursor.set_column(self.line_end_index(self.cursor.row));
            }
        }
    }
}
