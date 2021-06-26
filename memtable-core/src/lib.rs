use std::{
    collections::HashMap,
    iter::FromIterator,
    ops::{Index, IndexMut},
};

mod cell;
pub use cell::*;

#[cfg(feature = "csv")]
mod csv;

mod iter;
pub use iter::*;

mod position;
pub use position::*;

mod typed;
pub use typed::*;

/// Represents a table containing rows & columns of some data `T`
#[cfg_attr(feature = "serde-1", serde_with::serde_as)]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct Table<T> {
    /// Represents the table's data (cells) as a mapping between a cell's
    /// position and its actual content (private)
    #[cfg_attr(feature = "serde-1", serde_as("Vec<(_, _)>"))]
    cells: HashMap<Position, T>,

    /// Represents the total rows contained in the table based on the largest
    /// row position found
    row_cnt: usize,

    /// Represents the total columns contained in the table based on the largest
    /// column position found
    col_cnt: usize,
}

impl<T> Table<T> {
    /// Creates a new, empty table
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total rows contained in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.row_cnt(), 0);
    /// ```
    ///
    /// When has several rows:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    /// assert_eq!(table.row_cnt(), 2);
    /// ```
    ///
    pub fn row_cnt(&self) -> usize {
        self.row_cnt
    }

    /// Returns the total columns contained in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.col_cnt(), 0);
    /// ```
    ///
    /// When has several columns:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    /// assert_eq!(table.col_cnt(), 2);
    /// ```
    ///
    pub fn col_cnt(&self) -> usize {
        self.col_cnt
    }

    /// Returns the total cells (rows * columns) contained in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.len(), 0);
    /// ```
    ///
    /// When has several rows & columns:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    /// assert_eq!(table.len(), 6);
    /// ```
    ///
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Returns true if the total cells (rows * columns) contained in the table
    /// is zero
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert!(table.is_empty());
    /// ```
    ///
    /// When has several rows & columns:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    /// assert!(!table.is_empty());
    /// ```
    ///
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Returns an iterator of refs through all rows in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.rows().len(), 0);
    /// ```
    ///
    /// When has several rows:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut rows = table.rows();
    /// assert_eq!(rows.next().unwrap().copied().collect::<Vec<usize>>(), vec![1, 2, 3]);
    /// assert_eq!(rows.next().unwrap().copied().collect::<Vec<usize>>(), vec![4, 5, 6]);
    /// assert!(rows.next().is_none());
    /// ```
    ///
    pub fn rows(&self) -> Rows<T> {
        Rows::new(self)
    }

    /// Returns an iterator of refs through a specific row in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.row(0).len(), 0);
    /// ```
    ///
    /// When has several rows:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut cells = table.row(0);
    /// assert_eq!(cells.next().copied(), Some(1));
    /// assert_eq!(cells.next().copied(), Some(2));
    /// assert_eq!(cells.next().copied(), Some(3));
    /// assert_eq!(cells.next(), None);
    /// ```
    ///
    pub fn row(&self, idx: usize) -> Row<T> {
        Row::new(self, idx)
    }

    /// Consumes the table and returns an iterator through a specific row in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.into_row(0).len(), 0);
    /// ```
    ///
    /// When has several rows:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut cells = table.into_row(0);
    /// assert_eq!(cells.next(), Some(1));
    /// assert_eq!(cells.next(), Some(2));
    /// assert_eq!(cells.next(), Some(3));
    /// assert_eq!(cells.next(), None);
    /// ```
    ///
    pub fn into_row(self, idx: usize) -> IntoRow<T> {
        IntoRow::new(self, idx)
    }

    /// Returns an iterator of refs through all columns in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.columns().len(), 0);
    /// ```
    ///
    /// When has several columns:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// let mut columns = table.columns();
    /// assert_eq!(columns.next().unwrap().copied().collect::<Vec<usize>>(), vec![1, 2, 3]);
    /// assert_eq!(columns.next().unwrap().copied().collect::<Vec<usize>>(), vec![4, 5, 6]);
    /// assert!(columns.next().is_none());
    /// ```
    ///
    pub fn columns(&self) -> Columns<T> {
        Columns::new(self)
    }

    /// Returns an iterator of refs through a specific column in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.column(0).len(), 0);
    /// ```
    ///
    /// When has several columns:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// let mut cells = table.column(0);
    /// assert_eq!(cells.next().copied(), Some(1));
    /// assert_eq!(cells.next().copied(), Some(2));
    /// assert_eq!(cells.next().copied(), Some(3));
    /// assert_eq!(cells.next(), None);
    /// ```
    ///
    pub fn column(&self, idx: usize) -> Column<T> {
        Column::new(self, idx)
    }

    /// Consumes the table and returns an iterator through a specific column in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.into_column(0).len(), 0);
    /// ```
    ///
    /// When has several columns:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// let mut cells = table.into_column(0);
    /// assert_eq!(cells.next(), Some(1));
    /// assert_eq!(cells.next(), Some(2));
    /// assert_eq!(cells.next(), Some(3));
    /// assert_eq!(cells.next(), None);
    /// ```
    ///
    pub fn into_column(self, idx: usize) -> IntoColumn<T> {
        IntoColumn::new(self, idx)
    }

    /// Returns an iterator of refs through all cells in the table, starting
    /// from the first row, iterating through all cells from beginning to end,
    /// and then moving on to the next row
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.cells().len(), 0);
    /// ```
    ///
    /// When has several rows & columns:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut cells = table.cells();
    /// assert_eq!(cells.next(), Some(&1));
    /// assert_eq!(cells.next(), Some(&2));
    /// assert_eq!(cells.next(), Some(&3));
    /// assert_eq!(cells.next(), Some(&4));
    /// assert_eq!(cells.next(), Some(&5));
    /// assert_eq!(cells.next(), Some(&6));
    /// assert_eq!(cells.next(), None);
    /// ```
    ///
    pub fn cells(&self) -> Cells<T> {
        Cells::new(self)
    }

    /// Consumes the table and returns an iterator through all cells in the
    /// table, starting from the first row, iterating through all cells from
    /// beginning to end, and then moving on to the next row
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let table = Table::<usize>::new();
    /// assert_eq!(table.into_cells().len(), 0);
    /// ```
    ///
    /// When has several rows & columns:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut cells = table.into_cells();
    /// assert_eq!(cells.next(), Some(1));
    /// assert_eq!(cells.next(), Some(2));
    /// assert_eq!(cells.next(), Some(3));
    /// assert_eq!(cells.next(), Some(4));
    /// assert_eq!(cells.next(), Some(5));
    /// assert_eq!(cells.next(), Some(6));
    /// assert_eq!(cells.next(), None);
    /// ```
    ///
    pub fn into_cells(self) -> IntoCells<T> {
        IntoCells::new(self)
    }

    /// Returns whether or not a cell exists at the specified row & column. Note
    /// that this is not the same as whether or not the table's current row &
    /// column range would include a cell at that position! Rather, this is
    /// reporting if a cell actually exists
    ///
    /// ### Examples
    ///
    /// When has checking for a cell that doesn't exist:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert!(!table.has_cell(0, 3));
    /// ```
    ///
    /// When has checking for a cell that does exist:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert!(table.has_cell(0, 2));
    /// ```
    pub fn has_cell(&self, row: usize, col: usize) -> bool {
        self.cells.contains_key(&Position { row, col })
    }

    /// Returns reference to the cell found at the specified row and column
    ///
    /// ### Examples
    ///
    /// When retrieving a cell that doesn't exist:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert!(table.get_cell(0, 3).is_none());
    /// ```
    ///
    /// When retrieving a cell that does exist:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert_eq!(table.get_cell(0, 2), Some(&3));
    /// ```
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&T> {
        self.cells.get(&Position { row, col })
    }

    /// Returns mut reference to the cell found at the specified row and column
    ///
    /// ### Examples
    ///
    /// When retrieving a cell that doesn't exist:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert!(table.get_mut_cell(0, 3).is_none());
    /// ```
    ///
    /// When retrieving a cell that does exist:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    ///
    /// *table.get_mut_cell(0, 2).unwrap() = 999;
    /// assert_eq!(table.get_cell(0, 2), Some(&999));
    /// ```
    pub fn get_mut_cell(&mut self, row: usize, col: usize) -> Option<&mut T> {
        self.cells.get_mut(&Position { row, col })
    }

    /// Replaces the given value into the cell of the table at the specified
    /// row and column, returning the previous value contained in the cell
    ///
    /// ### Examples
    ///
    /// When replacing a cell that doesn't exist:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    ///
    /// assert!(table.insert_cell(0, 3, 999).is_none());
    /// assert_eq!(table.get_cell(0, 3), Some(&999));
    /// ```
    ///
    /// When replacing a cell that does exist:
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    ///
    /// assert_eq!(table.insert_cell(0, 2, 999), Some(3));
    /// assert_eq!(table.get_cell(0, 2), Some(&999));
    /// ```
    pub fn insert_cell(&mut self, row: usize, col: usize, value: T) -> Option<T> {
        // If cell exceeds current row range, adjust it
        if row >= self.row_cnt {
            self.row_cnt = row + 1;
        }

        // If cell exceeds current row range, adjust it
        if col >= self.col_cnt {
            self.col_cnt = col + 1;
        }

        self.cells.insert(Position { row, col }, value)
    }

    /// Removes the given value from the cell at the specified position
    ///
    /// Does not adjust the maximum row/column count within the table
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    ///
    /// assert_eq!(table.remove_cell(0, 0), Some(1));
    /// assert!(table.remove_cell(0, 0).is_none());
    /// ```
    pub fn remove_cell(&mut self, row: usize, col: usize) -> Option<T> {
        self.cells.remove(&Position { row, col })
    }

    /// Inserts a new row into the table at the given position, shifting down
    /// all rows after it
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// table.insert_row(0, vec![7, 8, 9]);
    ///
    /// let mut row = table.row(0);
    /// assert_eq!(row.next(), Some(&7));
    /// assert_eq!(row.next(), Some(&8));
    /// assert_eq!(row.next(), Some(&9));
    /// assert!(row.next().is_none());
    ///
    /// let mut row = table.row(1);
    /// assert_eq!(row.next(), Some(&1));
    /// assert_eq!(row.next(), Some(&2));
    /// assert_eq!(row.next(), Some(&3));
    /// assert!(row.next().is_none());
    ///
    /// let mut row = table.row(2);
    /// assert_eq!(row.next(), Some(&4));
    /// assert_eq!(row.next(), Some(&5));
    /// assert_eq!(row.next(), Some(&6));
    /// assert!(row.next().is_none());
    /// ```
    pub fn insert_row<I: IntoIterator<Item = T>>(&mut self, row: usize, cells: I) {
        // First, we need to shift down all cells that would appear at this
        // row or later
        if self.row_cnt > row {
            // NOTE: Need to go in reverse, otherwise we would overwrite the
            // row below when trying to shift down!
            for row in (row..self.row_cnt()).rev() {
                for col in (0..self.col_cnt()).rev() {
                    let pos = Position { row, col };
                    if let Some(x) = self.cells.remove(&pos) {
                        self.insert_cell(row + 1, col, x);
                    }
                }
            }
        }

        for (col, x) in cells.into_iter().enumerate() {
            self.insert_cell(row, col, x);
        }
    }

    /// Pushes a row to the end of the table
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut row = table.row(0);
    /// assert_eq!(row.next(), Some(&1));
    /// assert_eq!(row.next(), Some(&2));
    /// assert_eq!(row.next(), Some(&3));
    /// assert!(row.next().is_none());
    ///
    /// let mut row = table.row(1);
    /// assert_eq!(row.next(), Some(&4));
    /// assert_eq!(row.next(), Some(&5));
    /// assert_eq!(row.next(), Some(&6));
    /// assert!(row.next().is_none());
    /// ```
    pub fn push_row<I: IntoIterator<Item = T>>(&mut self, cells: I) {
        self.insert_row(self.row_cnt(), cells)
    }

    /// Removes the row at the specified position, shifting up all rows after it
    ///
    /// If the row does not exist, then an empty row will be returned
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut row = table.remove_row(0);
    /// assert_eq!(row.next(), Some(1));
    /// assert_eq!(row.next(), Some(2));
    /// assert_eq!(row.next(), Some(3));
    /// assert!(row.next().is_none());
    ///
    /// let mut row = table.remove_row(0);
    /// assert_eq!(row.next(), Some(4));
    /// assert_eq!(row.next(), Some(5));
    /// assert_eq!(row.next(), Some(6));
    /// assert!(row.next().is_none());
    ///
    /// let mut row = table.remove_row(0);
    /// assert!(row.next().is_none());
    /// ```
    pub fn remove_row(&mut self, row: usize) -> IntoRow<T> {
        // We will be storing the row into a temporary table that we then
        // convert into the iterator
        let mut tmp = Table::new();

        // First, we remove all cells in the specified row and add them to the
        // temporary table
        for col in 0..self.col_cnt() {
            if let Some(x) = self.remove_cell(row, col) {
                tmp.insert_cell(row, col, x);
            }
        }

        // Second, we need to shift up all cells that would appear after this row
        for row in (row + 1)..self.row_cnt() {
            for col in 0..self.col_cnt() {
                if let Some(x) = self.remove_cell(row, col) {
                    self.insert_cell(row - 1, col, x);
                }
            }
        }

        if self.row_cnt > 0 {
            self.row_cnt -= 1;
        }

        tmp.into_row(row)
    }

    /// Pops a row off the end of the table
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut row = table.pop_row();
    /// assert_eq!(row.next(), Some(4));
    /// assert_eq!(row.next(), Some(5));
    /// assert_eq!(row.next(), Some(6));
    /// assert!(row.next().is_none());
    ///
    /// let mut row = table.pop_row();
    /// assert_eq!(row.next(), Some(1));
    /// assert_eq!(row.next(), Some(2));
    /// assert_eq!(row.next(), Some(3));
    /// assert!(row.next().is_none());
    ///
    /// let mut row = table.pop_row();
    /// assert!(row.next().is_none());
    /// ```
    pub fn pop_row(&mut self) -> IntoRow<T> {
        let max_rows = self.row_cnt();
        self.remove_row(if max_rows > 0 { max_rows - 1 } else { 0 })
    }

    /// Inserts a new column into the table at the given position, shifting right
    /// all columns after it
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// table.insert_column(0, vec![7, 8, 9]);
    ///
    /// let mut column = table.column(0);
    /// assert_eq!(column.next(), Some(&7));
    /// assert_eq!(column.next(), Some(&8));
    /// assert_eq!(column.next(), Some(&9));
    /// assert!(column.next().is_none());
    ///
    /// let mut column = table.column(1);
    /// assert_eq!(column.next(), Some(&1));
    /// assert_eq!(column.next(), Some(&2));
    /// assert_eq!(column.next(), Some(&3));
    /// assert!(column.next().is_none());
    ///
    /// let mut column = table.column(2);
    /// assert_eq!(column.next(), Some(&4));
    /// assert_eq!(column.next(), Some(&5));
    /// assert_eq!(column.next(), Some(&6));
    /// assert!(column.next().is_none());
    /// ```
    pub fn insert_column<I: IntoIterator<Item = T>>(&mut self, col: usize, cells: I) {
        // First, we need to shift right all cells that would appear at this
        // column or later
        if self.col_cnt > col {
            // NOTE: Need to go in reverse, otherwise we would overwrite the
            // column right when trying to shift right!
            for row in (0..self.row_cnt()).rev() {
                for col in (col..self.col_cnt()).rev() {
                    if let Some(x) = self.remove_cell(row, col) {
                        self.insert_cell(row, col + 1, x);
                    }
                }
            }
        }

        for (row, x) in cells.into_iter().enumerate() {
            self.insert_cell(row, col, x);
        }
    }

    /// Pushes a column to the end of the table
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// let mut column = table.column(0);
    /// assert_eq!(column.next(), Some(&1));
    /// assert_eq!(column.next(), Some(&2));
    /// assert_eq!(column.next(), Some(&3));
    /// assert!(column.next().is_none());
    ///
    /// let mut column = table.column(1);
    /// assert_eq!(column.next(), Some(&4));
    /// assert_eq!(column.next(), Some(&5));
    /// assert_eq!(column.next(), Some(&6));
    /// assert!(column.next().is_none());
    /// ```
    pub fn push_column<I: IntoIterator<Item = T>>(&mut self, cells: I) {
        self.insert_column(self.col_cnt(), cells)
    }

    /// Removes the column at the specified position, shifting left all columns after it
    ///
    /// If the column does not exist, then an empty column will be returned
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// let mut column = table.remove_column(0);
    /// assert_eq!(column.next(), Some(1));
    /// assert_eq!(column.next(), Some(2));
    /// assert_eq!(column.next(), Some(3));
    /// assert!(column.next().is_none());
    ///
    /// let mut column = table.remove_column(0);
    /// assert_eq!(column.next(), Some(4));
    /// assert_eq!(column.next(), Some(5));
    /// assert_eq!(column.next(), Some(6));
    /// assert!(column.next().is_none());
    ///
    /// let mut column = table.remove_column(0);
    /// assert!(column.next().is_none());
    /// ```
    pub fn remove_column(&mut self, col: usize) -> IntoColumn<T> {
        // We will be storing the column into a temporary table that we then
        // convert into the iterator
        let mut tmp = Table::new();

        // First, we remove all cells in the specified row and add them to the
        // temporary table
        for row in 0..self.row_cnt() {
            if let Some(x) = self.remove_cell(row, col) {
                tmp.insert_cell(row, col, x);
            }
        }

        // Second, we need to shift up all cells that would appear after this row
        for row in 0..self.row_cnt() {
            for col in (col + 1)..self.col_cnt() {
                if let Some(x) = self.remove_cell(row, col) {
                    self.insert_cell(row, col - 1, x);
                }
            }
        }

        if self.col_cnt > 0 {
            self.col_cnt -= 1;
        }

        tmp.into_column(col)
    }

    /// Pops a column off the end of the table
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::Table;
    /// let mut table = Table::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// let mut column = table.pop_column();
    /// assert_eq!(column.next(), Some(4));
    /// assert_eq!(column.next(), Some(5));
    /// assert_eq!(column.next(), Some(6));
    /// assert!(column.next().is_none());
    ///
    /// let mut column = table.pop_column();
    /// assert_eq!(column.next(), Some(1));
    /// assert_eq!(column.next(), Some(2));
    /// assert_eq!(column.next(), Some(3));
    /// assert!(column.next().is_none());
    ///
    /// let mut column = table.pop_column();
    /// assert!(column.next().is_none());
    /// ```
    pub fn pop_column(&mut self) -> IntoColumn<T> {
        let max_cols = self.col_cnt();
        self.remove_column(if max_cols > 0 { max_cols - 1 } else { 0 })
    }

    /// Shrinks the table to fit where cells exist. This is only needed if you
    /// care about the row/column maximum size after removing all cells of a
    /// row or column individually.
    ///
    /// Runtime cost of O(MxN) where M is row count and N is column count
    pub fn shrink(&mut self) {
        let (max_row, max_col) = self.cells.keys().fold((0, 0), |acc, pos| {
            (
                std::cmp::max(acc.0, pos.row + 1),
                std::cmp::max(acc.1, pos.col + 1),
            )
        });

        self.row_cnt = max_row;
        self.col_cnt = max_col;
    }
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self {
            cells: HashMap::new(),
            row_cnt: 0,
            col_cnt: 0,
        }
    }
}

impl<'a, T> IntoIterator for &'a Table<T> {
    type Item = (Position, &'a T);
    type IntoIter = ZipPosition<Cells<'a, T>, &'a T>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.cells().zip_with_position()
    }
}

impl<T> IntoIterator for Table<T> {
    type Item = (Position, T);
    type IntoIter = ZipPosition<IntoCells<T>, T>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.into_cells().zip_with_position()
    }
}

impl<T, V: Into<T>> FromIterator<(usize, usize, V)> for Table<T> {
    /// Produces a table from the provided iterator of (row, col, value)
    fn from_iter<I: IntoIterator<Item = (usize, usize, V)>>(iter: I) -> Self {
        let cells: HashMap<Position, T> = iter
            .into_iter()
            .map(|(row, col, x)| (Position { row, col }, x.into()))
            .collect();
        Self::from(cells)
    }
}

impl<T, V: Into<T>> FromIterator<(Position, V)> for Table<T> {
    /// Produces a table from the provided iterator of (position, value)
    fn from_iter<I: IntoIterator<Item = (Position, V)>>(iter: I) -> Self {
        let cells: HashMap<Position, T> = iter.into_iter().map(|(p, x)| (p, x.into())).collect();
        Self::from(cells)
    }
}

impl<T> From<HashMap<Position, T>> for Table<T> {
    /// Creates a new table from the given hashmap of cells
    fn from(cells: HashMap<Position, T>) -> Self {
        let mut table = Self {
            cells,
            row_cnt: 0,
            col_cnt: 0,
        };

        // Shrink will calculate the proper row and column counts
        table.shrink();

        table
    }
}

impl<T> Index<(usize, usize)> for Table<T> {
    type Output = T;

    /// Indexes into a table by a specific row and column, returning a
    /// reference to the cell if it exists, otherwise panicking
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        self.get_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

impl<T> IndexMut<(usize, usize)> for Table<T> {
    /// Indexes into a table by a specific row and column, returning a mutable
    /// reference to the cell if it exists, otherwise panicking
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        self.get_mut_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_empty_hashmap<T>() -> HashMap<Position, T> {
        make_hashmap(Vec::new())
    }

    fn make_hashmap<T>(items: Vec<(usize, usize, T)>) -> HashMap<Position, T> {
        items
            .into_iter()
            .map(|(row, col, x)| (Position { row, col }, x))
            .collect()
    }

    #[test]
    fn new_should_calculate_row_and_column_counts_from_max_row_and_column() {
        let table = Table::from(make_empty_hashmap::<usize>());
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        let table = Table::from(make_hashmap(vec![(3, 2, "some value")]));
        assert_eq!(table.row_cnt(), 4);
        assert_eq!(table.col_cnt(), 3);

        let table = Table::from(make_hashmap(vec![(3, 0, "value"), (0, 5, "value")]));
        assert_eq!(table.row_cnt(), 4);
        assert_eq!(table.col_cnt(), 6);
    }

    #[test]
    fn get_cell_should_return_ref_to_cell_at_location() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
        ]));
        assert_eq!(table.get_cell(0, 0), Some(&"a"));
        assert_eq!(table.get_cell(0, 1), Some(&"b"));
        assert_eq!(table.get_cell(1, 0), Some(&"c"));
        assert_eq!(table.get_cell(1, 1), Some(&"d"));
        assert_eq!(table.get_cell(1, 2), None);
    }

    #[test]
    fn get_mut_cell_should_return_mut_ref_to_cell_at_location() {
        let mut table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
        ]));
        *table.get_mut_cell(0, 0).unwrap() = "e";
        assert_eq!(table.get_cell(0, 0), Some(&"e"));
    }

    #[test]
    fn insert_cell_should_extend_max_row_size_if_adding_beyond_max_row() {
        let mut table = Table::new();

        table.insert_cell(0, 0, "");
        table.insert_cell(0, 1, "");
        table.insert_cell(0, 2, "");
        assert_eq!(table.row_cnt(), 1);

        table.insert_cell(1, 0, "");
        assert_eq!(table.row_cnt(), 2);

        table.insert_cell(3, 0, "");
        assert_eq!(table.row_cnt(), 4);
    }

    #[test]
    fn insert_cell_should_extend_max_column_size_if_adding_beyond_max_column() {
        let mut table = Table::new();

        table.insert_cell(0, 0, "");
        table.insert_cell(1, 0, "");
        table.insert_cell(2, 0, "");
        assert_eq!(table.col_cnt(), 1);

        table.insert_cell(0, 1, "");
        assert_eq!(table.col_cnt(), 2);

        table.insert_cell(0, 3, "");
        assert_eq!(table.col_cnt(), 4);
    }

    #[test]
    fn insert_cell_should_return_previous_cell_and_overwrite_content() {
        let mut table = Table::new();

        assert!(table.insert_cell(0, 0, "test").is_none());
        assert_eq!(table.insert_cell(0, 0, "other"), Some("test"));
        assert_eq!(table.get_cell(0, 0), Some(&"other"))
    }

    #[test]
    fn remove_cell_should_return_cell_that_is_removed() {
        let mut table: Table<&'static str> = vec![(0, 0, "a"), (1, 1, "b")].into_iter().collect();

        assert_eq!(table.remove_cell(0, 0), Some("a"));
        assert!(table.remove_cell(0, 0).is_none());
    }

    #[test]
    fn insert_row_should_append_if_comes_after_last_row() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]
        .into_iter()
        .collect();

        table.insert_row(2, vec!["g", "h", "i"]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (1, 0, "d"),
                (1, 1, "e"),
                (1, 2, "f"),
                (2, 0, "g"),
                (2, 1, "h"),
                (2, 2, "i"),
            ]
        );
    }

    #[test]
    fn insert_row_should_shift_down_all_rows_on_or_after_specified_row() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]
        .into_iter()
        .collect();

        table.insert_row(1, vec!["g", "h", "i"]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (1, 0, "g"),
                (1, 1, "h"),
                (1, 2, "i"),
                (2, 0, "d"),
                (2, 1, "e"),
                (2, 2, "f"),
            ]
        );
    }

    #[test]
    fn insert_row_should_support_insertion_at_front() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]
        .into_iter()
        .collect();

        table.insert_row(0, vec!["g", "h", "i"]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "g"),
                (0, 1, "h"),
                (0, 2, "i"),
                (1, 0, "a"),
                (1, 1, "b"),
                (1, 2, "c"),
                (2, 0, "d"),
                (2, 1, "e"),
                (2, 2, "f"),
            ]
        );
    }

    #[test]
    fn push_row_should_insert_at_end() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]
        .into_iter()
        .collect();

        table.push_row(vec!["g", "h", "i"]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (1, 0, "d"),
                (1, 1, "e"),
                (1, 2, "f"),
                (2, 0, "g"),
                (2, 1, "h"),
                (2, 2, "i"),
            ]
        );
    }

    #[test]
    fn insert_column_should_append_if_comes_after_last_column() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]
        .into_iter()
        .collect();

        table.insert_column(3, vec!["g", "h"]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (0, 3, "g"),
                (1, 0, "d"),
                (1, 1, "e"),
                (1, 2, "f"),
                (1, 3, "h"),
            ]
        );
    }

    #[test]
    fn insert_column_should_shift_right_all_columns_on_or_after_specified_column() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]
        .into_iter()
        .collect();

        table.insert_column(1, vec!["g", "h"]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "g"),
                (0, 2, "b"),
                (0, 3, "c"),
                (1, 0, "d"),
                (1, 1, "h"),
                (1, 2, "e"),
                (1, 3, "f"),
            ]
        );
    }

    #[test]
    fn insert_column_should_support_insertion_at_front() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]
        .into_iter()
        .collect();

        table.insert_column(0, vec!["g", "h"]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "g"),
                (0, 1, "a"),
                (0, 2, "b"),
                (0, 3, "c"),
                (1, 0, "h"),
                (1, 1, "d"),
                (1, 2, "e"),
                (1, 3, "f"),
            ]
        );
    }

    #[test]
    fn push_column_should_insert_at_end() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]
        .into_iter()
        .collect();

        table.push_column(vec!["g", "h"]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (0, 3, "g"),
                (1, 0, "d"),
                (1, 1, "e"),
                (1, 2, "f"),
                (1, 3, "h"),
            ]
        );
    }

    #[test]
    fn remove_row_should_return_iterator_over_removed_row() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        let removed_cells: Vec<(usize, usize, &'static str)> = table
            .remove_row(1)
            .zip_with_position()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        assert_eq!(removed_cells, vec![(1, 0, "d"), (1, 1, "e"), (1, 2, "f")]);
    }

    #[test]
    fn remove_row_should_shift_rows_after_up() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        table.remove_row(1);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (1, 0, "g"),
                (1, 1, "h"),
                (1, 2, "i"),
            ]
        );
    }

    #[test]
    fn remove_row_should_support_removing_from_front() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        let removed_cells: Vec<(usize, usize, &'static str)> = table
            .remove_row(0)
            .zip_with_position()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        assert_eq!(removed_cells, vec![(0, 0, "a"), (0, 1, "b"), (0, 2, "c")]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "d"),
                (0, 1, "e"),
                (0, 2, "f"),
                (1, 0, "g"),
                (1, 1, "h"),
                (1, 2, "i"),
            ]
        );
    }

    #[test]
    fn remove_row_should_return_empty_iterator_if_row_missing() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        let removed_cells: Vec<(usize, usize, &'static str)> = table
            .remove_row(3)
            .zip_with_position()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        assert!(removed_cells.is_empty());

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (1, 0, "d"),
                (1, 1, "e"),
                (1, 2, "f"),
                (2, 0, "g"),
                (2, 1, "h"),
                (2, 2, "i"),
            ]
        );
    }

    #[test]
    fn pop_row_should_remove_last_row() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        let removed_cells: Vec<(usize, usize, &'static str)> = table
            .pop_row()
            .zip_with_position()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        assert_eq!(removed_cells, vec![(2, 0, "g"), (2, 1, "h"), (2, 2, "i")]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (1, 0, "d"),
                (1, 1, "e"),
                (1, 2, "f"),
            ]
        );
    }

    #[test]
    fn remove_column_should_return_iterator_over_removed_column() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        let removed_cells: Vec<(usize, usize, &'static str)> = table
            .remove_column(1)
            .zip_with_position()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        assert_eq!(removed_cells, vec![(0, 1, "b"), (1, 1, "e"), (2, 1, "h")]);
    }

    #[test]
    fn remove_column_should_shift_columns_after_left() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        table.remove_column(1);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "c"),
                (1, 0, "d"),
                (1, 1, "f"),
                (2, 0, "g"),
                (2, 1, "i"),
            ]
        );
    }

    #[test]
    fn remove_column_should_support_removing_from_front() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        let removed_cells: Vec<(usize, usize, &'static str)> = table
            .remove_column(0)
            .zip_with_position()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        assert_eq!(removed_cells, vec![(0, 0, "a"), (1, 0, "d"), (2, 0, "g")]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "b"),
                (0, 1, "c"),
                (1, 0, "e"),
                (1, 1, "f"),
                (2, 0, "h"),
                (2, 1, "i"),
            ]
        );
    }

    #[test]
    fn remove_column_should_return_empty_iterator_if_column_missing() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        let removed_cells: Vec<(usize, usize, &'static str)> = table
            .remove_column(3)
            .zip_with_position()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        assert!(removed_cells.is_empty());

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (1, 0, "d"),
                (1, 1, "e"),
                (1, 2, "f"),
                (2, 0, "g"),
                (2, 1, "h"),
                (2, 2, "i"),
            ]
        );
    }

    #[test]
    fn pop_column_should_remove_last_column() {
        let mut table: Table<&'static str> = vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]
        .into_iter()
        .collect();

        let removed_cells: Vec<(usize, usize, &'static str)> = table
            .pop_column()
            .zip_with_position()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        assert_eq!(removed_cells, vec![(0, 2, "c"), (1, 2, "f"), (2, 2, "i")]);

        let mut cells: Vec<(usize, usize, &'static str)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(
            cells,
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (1, 0, "d"),
                (1, 1, "e"),
                (2, 0, "g"),
                (2, 1, "h"),
            ]
        );
    }

    #[test]
    fn shrink_should_adjust_row_and_column_counts_based_on_cell_positions() {
        let mut table: Table<&'static str> = Table::new();
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        table.cells.insert(Position { row: 0, col: 3 }, "a");
        table.cells.insert(Position { row: 5, col: 0 }, "b");
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        table.shrink();
        assert_eq!(table.row_cnt(), 6);
        assert_eq!(table.col_cnt(), 4);
    }

    #[test]
    fn index_by_row_and_column_should_return_cell_ref() {
        let mut table = Table::new();
        table.push_row(vec![1, 2, 3]);

        assert_eq!(table[(0, 1)], 2);
    }

    #[test]
    #[should_panic]
    fn index_by_row_and_column_should_panic_if_cell_not_found() {
        let mut table = Table::new();
        table.push_row(vec![1, 2, 3]);

        let _ = table[(1, 0)];
    }

    #[test]
    fn index_mut_by_row_and_column_should_return_mutable_cell() {
        let mut table = Table::new();
        table.push_row(vec![1, 2, 3]);

        table[(0, 1)] = 999;

        let mut cells: Vec<(usize, usize, usize)> = table
            .cells
            .into_iter()
            .map(|(pos, x)| (pos.row, pos.col, x))
            .collect();
        cells.sort_unstable();
        assert_eq!(cells, vec![(0, 0, 1), (0, 1, 999), (0, 2, 3)]);
    }

    #[test]
    #[should_panic]
    fn index_mut_by_row_and_column_should_panic_if_cell_not_found() {
        let mut table = Table::new();
        table.push_row(vec![1, 2, 3]);

        table[(1, 0)] = 999;
    }
}
