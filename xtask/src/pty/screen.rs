//! An owned snapshot of the emulated terminal's cell grid.
//!
//! Owned rather than a borrow of the live parser: `wait_until` hands a
//! screen to a predicate while a reader thread is still feeding the
//! parser, and a borrow would deadlock or force predicates around a lock
//! guard.

/// A cell's color, mirroring the three forms a terminal can express.
///
/// Deliberately our own type rather than a re-export: a test asserting on
/// color should not have to name a parser crate's enum, and the driver
/// is free to change parsers without breaking every call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// The terminal's default foreground/background.
    Default,
    /// One of the 256 palette entries.
    Indexed(u8),
    /// A direct-color triple.
    Rgb(u8, u8, u8),
}

/// One cell of the grid: what is drawn there and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The cell's contents. Empty for an untouched cell, and possibly
    /// several `char`s long for a base character plus combining marks.
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

impl Cell {
    /// The character actually painted, with an untouched cell reading as
    /// a space so line text lines up with what a human sees.
    pub fn display(&self) -> &str {
        if self.text.is_empty() {
            " "
        } else {
            &self.text
        }
    }
}

/// A full screen snapshot, addressed zero-based as `(col, row)`.
#[derive(Debug, Clone)]
pub struct Screen {
    cols: u16,
    rows: u16,
    cells: Vec<Cell>,
}

impl Screen {
    pub fn from_cells(cols: u16, rows: u16, cells: Vec<Cell>) -> Self {
        assert_eq!(cells.len(), cols as usize * rows as usize);
        Self { cols, rows, cells }
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    pub fn cell(&self, col: u16, row: u16) -> Option<&Cell> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        self.cells
            .get(row as usize * self.cols as usize + col as usize)
    }

    /// One row rendered as text, trailing blanks trimmed.
    pub fn line(&self, row: u16) -> String {
        self.line_with_columns(row).0
    }

    /// `line(row)`, plus the cell column each byte offset in it belongs to.
    /// Built in one walk because they must agree: a cell can hold several
    /// `char`s (a base plus combining marks) and a double-width glyph's
    /// continuation cell is blank, so neither a byte index nor a `char`
    /// index is a column, and `find` returns columns that get clicked.
    fn line_with_columns(&self, row: u16) -> (String, Vec<u16>) {
        if row >= self.rows {
            return (String::new(), Vec::new());
        }
        let start = row as usize * self.cols as usize;
        let mut line = String::new();
        let mut columns = Vec::new();
        for (offset, cell) in self.cells[start..start + self.cols as usize]
            .iter()
            .enumerate()
        {
            let text = cell.display();
            line.push_str(text);
            // `repeat_n` would read better but only stabilised in 1.82,
            // and this workspace's MSRV is 1.80.
            columns.extend(std::iter::repeat(offset as u16).take(text.len()));
        }
        let trimmed = line.trim_end();
        columns.truncate(trimmed.len());
        (trimmed.to_string(), columns)
    }

    /// The whole screen as text, one `\n`-separated line per row.
    pub fn text(&self) -> String {
        (0..self.rows)
            .map(|row| self.line(row))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.find(needle).is_some()
    }

    /// Zero-based `(col, row)` of the first occurrence of `needle`, scanning
    /// top to bottom then left to right. The column is a *cell* index (what
    /// the mouse methods want), so a double-width glyph counts once.
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        self.find_all(needle).into_iter().next()
    }

    /// Every occurrence of `needle`, at most one per row.
    pub fn find_all(&self, needle: &str) -> Vec<(u16, u16)> {
        let mut hits = Vec::new();
        for row in 0..self.rows {
            let (line, columns) = self.line_with_columns(row);
            if let Some(byte_offset) = line.find(needle) {
                // `str::find` gives a byte offset; a cell column is what
                // callers click with, and the two differ the moment the
                // line holds a non-ASCII glyph -- which every pane
                // border and ref pill in this TUI does.
                if let Some(&col) = columns.get(byte_offset) {
                    hits.push((col, row));
                }
            }
        }
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(text: &str) -> Cell {
        Cell {
            text: text.to_string(),
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            italic: false,
            underline: false,
            inverse: false,
        }
    }

    /// Two rows: "ab " / " ab".
    fn fixture() -> Screen {
        Screen::from_cells(
            3,
            2,
            vec![
                plain("a"),
                plain("b"),
                plain(""),
                plain(""),
                plain("a"),
                plain("b"),
            ],
        )
    }

    #[test]
    fn a_line_reads_left_to_right_with_untouched_cells_as_spaces() {
        let screen = fixture();
        assert_eq!(screen.line(0), "ab");
        assert_eq!(screen.line(1), " ab");
        assert_eq!(screen.text(), "ab\n ab");
    }

    #[test]
    fn find_reports_the_column_the_needle_starts_at() {
        let screen = fixture();
        assert_eq!(screen.find("ab"), Some((0, 0)));
        assert_eq!(screen.find("b"), Some((1, 0)));
        assert_eq!(screen.find_all("ab"), vec![(0, 0), (1, 1)]);
        assert_eq!(screen.find("zz"), None);
    }

    /// A cell may hold several `char`s, and a double-width glyph's
    /// continuation cell is blank - so neither a byte index nor a char
    /// index is a column. This is the case that makes `find`'s result
    /// usable as the argument to `click`.
    #[test]
    fn find_returns_a_cell_column_even_past_a_wide_glyph() {
        // A wide glyph plus its blank continuation cell, then "ok".
        let screen = Screen::from_cells(
            4,
            1,
            vec![plain("\u{1f500}"), plain(""), plain("o"), plain("k")],
        );
        assert_eq!(screen.line(0), "\u{1f500} ok");
        assert_eq!(screen.find("ok"), Some((2, 0)));
    }

    #[test]
    fn out_of_range_cells_are_none_rather_than_a_panic() {
        let screen = fixture();
        assert_eq!(screen.cell(0, 0).map(Cell::display), Some("a"));
        assert!(screen.cell(3, 0).is_none());
        assert!(screen.cell(0, 2).is_none());
    }
}
