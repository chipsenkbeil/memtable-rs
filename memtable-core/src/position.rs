use std::cmp::Ordering;

/// Represents the position of a cell in a table
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    /// Represents the row number of a cell starting from 0
    pub row: usize,

    /// Represents the coumn number of a cell starting from 0
    pub col: usize,
}

impl Position {
    /// Creates a new position with the given row and column
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl PartialOrd for Position {
    /// Compares positions in terms of order by seeing if one comes before/after
    /// another in rows. If on the same row, then the columns are compared.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    /// Compares positions in terms of order by seeing if one comes before/after
    /// another in rows. If on the same row, then the columns are compared.
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Position;
    /// // Row is first used for comparisons
    /// assert!(Position { row: 0, col: 1 } < Position { row: 1, col: 0 });
    /// assert!(Position { row: 1, col: 0 } > Position { row: 0, col: 1 });
    ///
    /// // Column is used for comparisons if rows are equal
    /// assert!(Position { row: 0, col: 0 } < Position { row: 0, col: 1 });
    /// assert!(Position { row: 0, col: 1 } > Position { row: 0, col: 0 });
    ///
    /// // Row & column need to match for equality
    /// assert_eq!(Position { row: 0, col: 0 }, Position { row: 0, col: 0 });
    /// ```
    fn cmp(&self, other: &Self) -> Ordering {
        match self.row.cmp(&other.row) {
            Ordering::Equal => self.col.cmp(&other.col),
            x => x,
        }
    }
}
