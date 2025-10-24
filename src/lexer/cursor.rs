#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    /// Line number
    pub line: usize,
    /// Column number
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor { line: 0, col: 0 }
    }

    /// Set line and column
    pub fn set(&mut self, line: usize, col: usize) {
        self.line = line;
        self.col = col;
    }

    /// Advance one line and sets column to 0
    pub fn next_line(&mut self) {
        self.line += 1;
        self.col = 0;
    }

    /// Advance one column
    pub fn next_col(&mut self) {
        self.col += 1;
    }
}
