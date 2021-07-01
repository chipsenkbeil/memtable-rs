//! # memtable-core
//!
//! Provides the core structs and traits for use in table manipulation.
//!
//! Check out full documentation at
//! [memtable](https://github.com/chipsenkbeil/memtable-rs).
#![cfg_attr(feature = "docs", feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

/// Contains extensions to the library based on extra features
pub mod exts;

/// Contains iterators and associated traits for traversing portions of tables
pub mod iter;

mod impls;
pub use impls::*;

pub mod list;

mod position;

#[doc(inline)]
pub use position::Position;

/// Contains relevant top-level traits, structs, and more to make use of
/// this library
pub mod prelude;

mod utils;

// Re-export alloc as std in the case where we don't have std
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as std;

/// Represents an abstract table of data
pub trait Table: Sized {
    /// The type of data stored in individual cells within the table
    type Data;

    /// The type of structure to hold a row of data
    type Row: list::List<Item = Self::Data>;

    /// The type of structure to hold a column of data
    type Column: list::List<Item = Self::Data>;

    /// Returns the total rows contained in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.row_cnt(), 0);
    /// ```
    ///
    /// When has several rows:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    /// assert_eq!(table.row_cnt(), 2);
    /// ```
    ///
    fn row_cnt(&self) -> usize;

    /// Returns the total columns contained in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.col_cnt(), 0);
    /// ```
    ///
    /// When has several columns:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    /// assert_eq!(table.col_cnt(), 2);
    /// ```
    ///
    fn col_cnt(&self) -> usize;

    /// Sets the preferred capacity of the table when it comes to total rows
    ///
    /// This is a preference, not an absolute, and is up to each table to
    /// implement if desired; otherwise, this does nothing by default
    #[allow(unused_variables)]
    fn set_row_capacity(&mut self, capacity: usize) {}

    /// Sets the preferred capacity of the table when it comes to total columns
    ///
    /// This is a preference, not an absolute, and is up to each table to
    /// implement if desired; otherwise, this does nothing by default
    #[allow(unused_variables)]
    fn set_column_capacity(&mut self, capacity: usize) {}

    /// Returns reference to the cell found at the specified row and column
    ///
    /// ### Examples
    ///
    /// When retrieving a cell that doesn't exist:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert!(table.get_cell(0, 3).is_none());
    /// ```
    ///
    /// When retrieving a cell that does exist:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert_eq!(table.get_cell(0, 2), Some(&3));
    /// ```
    fn get_cell(&self, row: usize, col: usize) -> Option<&Self::Data>;

    /// Returns mut reference to the cell found at the specified row and column
    ///
    /// ### Examples
    ///
    /// When retrieving a cell that doesn't exist:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert!(table.get_mut_cell(0, 3).is_none());
    /// ```
    ///
    /// When retrieving a cell that does exist:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    ///
    /// *table.get_mut_cell(0, 2).unwrap() = 999;
    /// assert_eq!(table.get_cell(0, 2), Some(&999));
    /// ```
    fn get_mut_cell(&mut self, row: usize, col: usize) -> Option<&mut Self::Data>;

    /// Replaces the given value into the cell of the table at the specified
    /// row and column, returning the previous value contained in the cell
    ///
    /// ### Examples
    ///
    /// When replacing a cell that doesn't exist:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    ///
    /// assert!(table.insert_cell(0, 3, 999).is_none());
    /// assert_eq!(table.get_cell(0, 3), Some(&999));
    /// ```
    ///
    /// When replacing a cell that does exist:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    ///
    /// assert_eq!(table.insert_cell(0, 2, 999), Some(3));
    /// assert_eq!(table.get_cell(0, 2), Some(&999));
    /// ```
    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data>;

    /// Removes the given value from the cell at the specified position, but
    /// does not shift any other cell to fill in the gap
    ///
    /// Does not attempt to adjust the capacity within the table
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    ///
    /// assert_eq!(table.remove_cell(0, 0), Some(1));
    /// assert!(table.remove_cell(0, 0).is_none());
    /// ```
    fn remove_cell(&mut self, row: usize, col: usize) -> Option<Self::Data>;

    /// Returns the total cells (rows * columns) contained in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.len(), 0);
    /// ```
    ///
    /// When has several rows & columns:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    /// assert_eq!(table.len(), 6);
    /// ```
    ///
    fn len(&self) -> usize {
        self.row_cnt() * self.col_cnt()
    }

    /// Returns true if the total cells (rows * columns) contained in the table
    /// is zero
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert!(table.is_empty());
    /// ```
    ///
    /// When has several rows & columns:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    /// assert!(!table.is_empty());
    /// ```
    ///
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator of refs through all rows in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.rows().len(), 0);
    /// ```
    ///
    /// When has several rows:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// let mut rows = table.rows();
    /// assert_eq!(rows.next().unwrap().copied().collect::<Vec<usize>>(), vec![1, 2, 3]);
    /// assert_eq!(rows.next().unwrap().copied().collect::<Vec<usize>>(), vec![4, 5, 6]);
    /// assert!(rows.next().is_none());
    /// ```
    ///
    fn rows(&self) -> iter::Rows<Self::Data, Self> {
        iter::Rows::new(self)
    }

    /// Returns an iterator of refs through a specific row in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.row(0).len(), 0);
    /// ```
    ///
    /// When has several rows:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
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
    fn row(&self, idx: usize) -> iter::Row<Self::Data, Self> {
        iter::Row::new(self, idx)
    }

    /// Consumes the table and returns an iterator through a specific row in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.into_row(0).len(), 0);
    /// ```
    ///
    /// When has several rows:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
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
    fn into_row(self, idx: usize) -> iter::IntoRow<Self::Data, Self> {
        iter::IntoRow::new(self, idx)
    }

    /// Returns an iterator of refs through all columns in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.columns().len(), 0);
    /// ```
    ///
    /// When has several columns:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// let mut columns = table.columns();
    /// assert_eq!(columns.next().unwrap().copied().collect::<Vec<usize>>(), vec![1, 2, 3]);
    /// assert_eq!(columns.next().unwrap().copied().collect::<Vec<usize>>(), vec![4, 5, 6]);
    /// assert!(columns.next().is_none());
    /// ```
    ///
    fn columns(&self) -> iter::Columns<Self::Data, Self> {
        iter::Columns::new(self)
    }

    /// Returns an iterator of refs through a specific column in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.column(0).len(), 0);
    /// ```
    ///
    /// When has several columns:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
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
    fn column(&self, idx: usize) -> iter::Column<Self::Data, Self> {
        iter::Column::new(self, idx)
    }

    /// Consumes the table and returns an iterator through a specific column in the table
    ///
    /// ### Examples
    ///
    /// When empty:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.into_column(0).len(), 0);
    /// ```
    ///
    /// When has several columns:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
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
    fn into_column(self, idx: usize) -> iter::IntoColumn<Self::Data, Self> {
        iter::IntoColumn::new(self, idx)
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
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.cells().len(), 0);
    /// ```
    ///
    /// When has several rows & columns:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
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
    fn cells(&self) -> iter::Cells<Self::Data, Self> {
        iter::Cells::new(self)
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
    /// # use memtable_core::prelude::*;
    /// let table = DynamicTable::<usize>::new();
    /// assert_eq!(table.into_cells().len(), 0);
    /// ```
    ///
    /// When has several rows & columns:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::<usize>::new();
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
    fn into_cells(self) -> iter::IntoCells<Self::Data, Self> {
        iter::IntoCells::new(self)
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
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert!(!table.has_cell(0, 3));
    /// ```
    ///
    /// When has checking for a cell that does exist:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    /// assert!(table.has_cell(0, 2));
    /// ```
    fn has_cell(&self, row: usize, col: usize) -> bool {
        self.get_cell(row, col).is_some()
    }

    /// Inserts a new row into the table at the given position, shifting down
    /// all rows after it
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
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
    fn insert_row<I: IntoIterator<Item = Self::Data>>(&mut self, row: usize, cells: I) {
        // First, we need to shift down all cells that would appear at this
        // row or later
        if self.row_cnt() > row {
            // NOTE: Need to go in reverse, otherwise we would overwrite the
            // row below when trying to shift down!
            for row in (row..self.row_cnt()).rev() {
                for col in (0..self.col_cnt()).rev() {
                    if let Some(x) = self.remove_cell(row, col) {
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
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
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
    fn push_row<I: IntoIterator<Item = Self::Data>>(&mut self, cells: I) {
        self.insert_row(self.row_cnt(), cells)
    }

    /// Removes the row at the specified position, shifting up all rows after it
    ///
    /// If the row does not exist, then an empty row will be returned
    ///
    /// ### Examples
    ///
    /// Removing from the front:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// assert_eq!(table.remove_row(0), Some(DynamicList::from([1, 2, 3])));
    /// assert_eq!(table.remove_row(0), Some(DynamicList::from([4, 5, 6])));
    /// assert_eq!(table.remove_row(0), None);
    /// ```
    ///
    /// Removing from the back:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// assert_eq!(table.remove_row(1), Some(DynamicList::from([4, 5, 6])));
    /// assert_eq!(table.remove_row(1), None);
    /// assert_eq!(table.remove_row(0), Some(DynamicList::from([1, 2, 3])));
    /// assert_eq!(table.remove_row(0), None);
    /// ```
    fn remove_row(&mut self, row: usize) -> Option<Self::Row> {
        let row_cnt = self.row_cnt();
        let col_cnt = self.col_cnt();

        // If not in table range, return none
        if row >= row_cnt {
            return None;
        }

        // First, we remove all cells in the specified row and add them to the
        // temporary table
        use list::List;
        let tmp = Self::Row::new_filled_with(col_cnt, |col| self.remove_cell(row, col));

        // Second, we need to shift up all cells that would appear after this row
        for row in (row + 1)..row_cnt {
            for col in 0..col_cnt {
                if let Some(x) = self.remove_cell(row, col) {
                    self.insert_cell(row - 1, col, x);
                }
            }
        }

        // Flag to table that the preferred row capacity is now one less
        // if the row we removed was within capacity
        if row < row_cnt {
            self.set_row_capacity(row_cnt - 1);
        }

        Some(tmp)
    }

    /// Pops a row off the end of the table
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_row(vec![1, 2, 3]);
    /// table.push_row(vec![4, 5, 6]);
    ///
    /// assert_eq!(table.pop_row(), Some(DynamicList::from([4, 5, 6])));
    /// assert_eq!(table.pop_row(), Some(DynamicList::from([1, 2, 3])));
    /// assert_eq!(table.pop_row(), None);
    /// ```
    fn pop_row(&mut self) -> Option<Self::Row> {
        let max_rows = self.row_cnt();
        self.remove_row(if max_rows > 0 { max_rows - 1 } else { 0 })
    }

    /// Inserts a new column into the table at the given position, shifting right
    /// all columns after it
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
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
    fn insert_column<I: IntoIterator<Item = Self::Data>>(&mut self, col: usize, cells: I) {
        // First, we need to shift right all cells that would appear at this
        // column or later
        if self.col_cnt() > col {
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
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
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
    fn push_column<I: IntoIterator<Item = Self::Data>>(&mut self, cells: I) {
        self.insert_column(self.col_cnt(), cells)
    }

    /// Removes the column at the specified position, shifting left all columns after it
    ///
    /// If the column does not exist, then an empty column will be returned
    ///
    /// ### Examples
    ///
    /// Removing from the front:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// assert_eq!(table.remove_column(0), Some(DynamicList::from([1, 2, 3])));
    /// assert_eq!(table.remove_column(0), Some(DynamicList::from([4, 5, 6])));
    /// assert_eq!(table.remove_column(0), None);
    /// ```
    ///
    /// Removing from the the back:
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// assert_eq!(table.remove_column(1), Some(DynamicList::from([4, 5, 6])));
    /// assert_eq!(table.remove_column(1), None);
    /// assert_eq!(table.remove_column(0), Some(DynamicList::from([1, 2, 3])));
    /// assert_eq!(table.remove_column(0), None);
    /// ```
    fn remove_column(&mut self, col: usize) -> Option<Self::Column> {
        let row_cnt = self.row_cnt();
        let col_cnt = self.col_cnt();

        // If not in table range, return none
        if col >= col_cnt {
            return None;
        }

        // First, we remove all cells in the specified column and add them to the
        // temporary table
        use list::List;
        let tmp = Self::Column::new_filled_with(row_cnt, |row| self.remove_cell(row, col));

        // Second, we need to shift left all cells that would appear after this column
        for row in 0..row_cnt {
            for col in (col + 1)..col_cnt {
                if let Some(x) = self.remove_cell(row, col) {
                    self.insert_cell(row, col - 1, x);
                }
            }
        }

        // Flag to table that the preferred column capacity is now one less
        // if the column we removed was within capacity
        if col < col_cnt {
            self.set_column_capacity(col_cnt - 1);
        }

        Some(tmp)
    }

    /// Pops a column off the end of the table
    ///
    /// ### Examples
    ///
    /// ```
    /// # use memtable_core::prelude::*;
    /// let mut table = DynamicTable::new();
    /// table.push_column(vec![1, 2, 3]);
    /// table.push_column(vec![4, 5, 6]);
    ///
    /// assert_eq!(table.pop_column(), Some(DynamicList::from([4, 5, 6])));
    /// assert_eq!(table.pop_column(), Some(DynamicList::from([1, 2, 3])));
    /// assert_eq!(table.pop_column(), None);
    /// ```
    fn pop_column(&mut self) -> Option<Self::Column> {
        let max_cols = self.col_cnt();
        self.remove_column(if max_cols > 0 { max_cols - 1 } else { 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // For a couple of tests, we also provide a dummy table with no actual data
    #[derive(Default)]
    struct DummyTable {
        row_cnt: usize,
        col_cnt: usize,
        last_requested_row_capacity: Option<usize>,
        last_requested_column_capacity: Option<usize>,
    }

    impl DummyTable {
        pub fn new(row_cnt: usize, col_cnt: usize) -> Self {
            Self {
                row_cnt,
                col_cnt,
                ..Default::default()
            }
        }
    }

    impl Table for DummyTable {
        type Data = ();
        type Row = list::FixedList<Self::Data, 0>;
        type Column = list::FixedList<Self::Data, 0>;

        fn set_row_capacity(&mut self, row: usize) {
            self.last_requested_row_capacity = Some(row);
        }
        fn set_column_capacity(&mut self, col: usize) {
            self.last_requested_column_capacity = Some(col);
        }
        fn row_cnt(&self) -> usize {
            self.row_cnt
        }
        fn col_cnt(&self) -> usize {
            self.col_cnt
        }
        fn get_cell(&self, _row: usize, _col: usize) -> Option<&Self::Data> {
            None
        }
        fn get_mut_cell(&mut self, _row: usize, _col: usize) -> Option<&mut Self::Data> {
            None
        }
        fn insert_cell(
            &mut self,
            _row: usize,
            _col: usize,
            _value: Self::Data,
        ) -> Option<Self::Data> {
            None
        }
        fn remove_cell(&mut self, _row: usize, _col: usize) -> Option<Self::Data> {
            None
        }
    }

    #[test]
    fn remove_row_should_set_new_row_capacity_if_valid_row_removed() {
        let mut table = DummyTable::new(2, 0);
        assert_eq!(table.last_requested_row_capacity, None);

        // Remove out of range, so should not call
        table.remove_row(2);
        assert_eq!(table.last_requested_row_capacity, None);

        // Remove in range, so should call
        table.remove_row(1);
        assert_eq!(table.last_requested_row_capacity, Some(1));
    }

    #[test]
    fn remove_column_should_set_new_column_capacity_if_valid_column_removed() {
        let mut table = DummyTable::new(0, 2);
        assert_eq!(table.last_requested_column_capacity, None);

        // Remove out of range, so should not call
        table.remove_column(2);
        assert_eq!(table.last_requested_column_capacity, None);

        // Remove in range, so should call
        table.remove_column(1);
        assert_eq!(table.last_requested_column_capacity, Some(1));
    }
}
