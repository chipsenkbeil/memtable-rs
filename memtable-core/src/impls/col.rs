use crate::{iter::*, utils, Position, Table};
use std::{
    cmp,
    iter::FromIterator,
    mem,
    ops::{Index, IndexMut},
};

/// Represents an inmemory table containing rows & columns of some data `T`
/// with a fixed capacity across columns, but ability to grow dynamically with
/// rows
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct FixedColumnTable<T: Default, const COL: usize> {
    /// Internal allocation of our table's data
    #[cfg_attr(
        feature = "serde-1",
        serde(
            bound(
                serialize = "T: serde::Serialize",
                deserialize = "T: serde::Deserialize<'de>"
            ),
            serialize_with = "utils::serialize_vec_array",
            deserialize_with = "utils::deserialize_vec_array"
        )
    )]
    cells: Vec<[T; COL]>,

    /// Represents a tracker for how many rows out of our total capacity
    /// have been used
    row_cnt: usize,

    /// Represents a tracker for how many columns out of our total capacity
    /// have been used
    col_cnt: usize,
}

impl<T: Default, const COL: usize> FixedColumnTable<T, COL> {
    /// Creates a new, empty table
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes all cells contained within the table that are outside the
    /// current row capacity
    pub fn truncate(&mut self) {
        // Shrink to row_cnt total rows
        self.cells.truncate(self.row_cnt);

        // Now go through each row and re-assign columns to default values
        for row in self.cells.iter_mut() {
            for cell in row[self.col_cnt..COL].iter_mut() {
                *cell = T::default();
            }
        }
    }

    /// Returns an iterator over the cells and their positions within the table
    pub fn iter(&self) -> ZipPosition<&T, Cells<T, FixedColumnTable<T, COL>>> {
        self.into_iter()
    }
}

impl<T: Default, const COL: usize> Table for FixedColumnTable<T, COL> {
    type Data = T;

    fn row_cnt(&self) -> usize {
        self.row_cnt
    }

    fn col_cnt(&self) -> usize {
        self.col_cnt
    }

    fn get_cell(&self, row: usize, col: usize) -> Option<&Self::Data> {
        if row < self.row_cnt && col < self.col_cnt {
            Some(&self.cells[row][col])
        } else {
            None
        }
    }

    fn get_mut_cell(&mut self, row: usize, col: usize) -> Option<&mut Self::Data> {
        if row < self.row_cnt && col < self.col_cnt {
            Some(&mut self.cells[row][col])
        } else {
            None
        }
    }

    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data> {
        // Allow inserting anywhere in the allocated space, not just virtual
        if col < COL {
            let mut did_grow = false;
            if row >= self.row_cnt {
                self.cells.resize_with(row + 1, utils::default_array);
                self.row_cnt = row + 1;
                did_grow = true;
            }

            if col >= self.col_cnt {
                self.col_cnt = col + 1;
                did_grow = true;
            }

            // Perform operation, but if growing our virtual range, don't
            // return anything and pretend that it was empty
            let old_value = mem::replace(&mut self.cells[row][col], value);
            if !did_grow {
                Some(old_value)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn remove_cell(&mut self, row: usize, col: usize) -> Option<Self::Data> {
        // TODO: Same problem as elsewhere, how do we know when to shrink our
        //       row and col counts? Especially, unlike the dynamic scenario,
        //       we can't rely on values not being in a map to determine
        self.insert_cell(row, col, T::default())
    }

    /// Will adjust the internal row count tracker to the specified capacity
    ///
    /// Note that this does **not** remove any cells from the table in their
    /// old positions. Instead, this updates the virtual space within the
    /// table that is made available for methods like [`Table::get_cell`].
    ///
    /// If you want to remove the cells that are no longer within capacity,
    /// call [`Self::truncate`], which will reset them to their default value.
    fn set_row_capacity(&mut self, capacity: usize) {
        self.row_cnt = capacity;
    }

    /// Will adjust the internal column count tracker to the specified capacity,
    /// capping at COL.
    ///
    /// Note that this does **not** remove any cells from the table in their
    /// old positions. Instead, this updates the virtual space within the
    /// table that is made available for methods like [`Table::get_cell`].
    ///
    /// If you want to remove the cells that are no longer within capacity,
    /// call [`Self::truncate`], which will reset them to their default value.
    fn set_column_capacity(&mut self, capacity: usize) {
        self.col_cnt = cmp::min(capacity, COL);
    }
}

impl<T: Default, const COL: usize> From<Vec<[T; COL]>> for FixedColumnTable<T, COL> {
    /// Creates a new table with maximum allocation of COL for each row, assuming
    /// all provided columns have been filled
    ///
    /// If this is incorrect, adjust the virtual row and column counts with
    /// [`Table::set_row_capacity`] and [`Table::set_column_capacity`] respectively.
    fn from(cells: Vec<[T; COL]>) -> Self {
        let row_cnt = cells.len();
        Self {
            cells,
            row_cnt,
            col_cnt: COL,
        }
    }
}

impl<'a, T: Default, const COL: usize> IntoIterator for &'a FixedColumnTable<T, COL> {
    type Item = (Position, &'a T);
    type IntoIter = ZipPosition<&'a T, Cells<'a, T, FixedColumnTable<T, COL>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.cells().zip_with_position()
    }
}

impl<T: Default, const COL: usize> IntoIterator for FixedColumnTable<T, COL> {
    type Item = (Position, T);
    type IntoIter = ZipPosition<T, IntoCells<T, FixedColumnTable<T, COL>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.into_cells().zip_with_position()
    }
}

impl<T: Default, V: Into<T>, const COL: usize> FromIterator<(usize, usize, V)>
    for FixedColumnTable<T, COL>
{
    /// Produces a table from the provided iterator of (row, col, value). All
    /// values that would go outside of the range of the table will be dropped.
    fn from_iter<I: IntoIterator<Item = (usize, usize, V)>>(iter: I) -> Self {
        let mut table = Self::new();
        for (row, col, value) in iter.into_iter() {
            table.insert_cell(row, col, value.into());
        }
        table
    }
}

impl<T: Default, V: Into<T>, const COL: usize> FromIterator<(Position, V)>
    for FixedColumnTable<T, COL>
{
    /// Produces a table from the provided iterator of (position, value). All
    /// values that would go outside of the range of the table will be dropped.
    fn from_iter<I: IntoIterator<Item = (Position, V)>>(iter: I) -> Self {
        let mut table = Self::new();
        for (pos, value) in iter.into_iter() {
            table.insert_cell(pos.row, pos.col, value.into());
        }
        table
    }
}

impl<T: Default, const COL: usize> Index<(usize, usize)> for FixedColumnTable<T, COL> {
    type Output = T;

    /// Indexes into a table by a specific row and column, returning a
    /// reference to the cell if it exists, otherwise panicking
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        self.get_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

impl<T: Default, const COL: usize> IndexMut<(usize, usize)> for FixedColumnTable<T, COL> {
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

    #[test]
    fn row_cnt_should_be_adjustable() {
        let mut table: FixedColumnTable<usize, 0> = FixedColumnTable::new();
        assert_eq!(table.row_cnt(), 0);
        table.set_row_capacity(999);
        assert_eq!(table.row_cnt(), 999);

        let mut table: FixedColumnTable<usize, 4> = FixedColumnTable::new();
        assert_eq!(table.row_cnt(), 0);
        table.set_row_capacity(999);
        assert_eq!(table.row_cnt(), 999);
    }

    #[test]
    fn col_cnt_should_be_adjustable_up_to_const_max() {
        let mut table: FixedColumnTable<usize, 0> = FixedColumnTable::new();
        assert_eq!(table.col_cnt(), 0);
        table.set_column_capacity(1);
        assert_eq!(table.col_cnt(), 0);

        let mut table: FixedColumnTable<usize, 4> = FixedColumnTable::new();
        assert_eq!(table.col_cnt(), 0);
        table.set_column_capacity(5);
        assert_eq!(table.col_cnt(), 4);
    }

    #[test]
    fn get_cell_should_return_ref_to_cell_at_location() {
        // Sets capacity to that of the 2D array provided
        let table = FixedColumnTable::from(vec![["a", "b"], ["c", "d"]]);
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.get_cell(0, 1).as_deref(), Some(&"b"));
        assert_eq!(table.get_cell(1, 0).as_deref(), Some(&"c"));
        assert_eq!(table.get_cell(1, 1).as_deref(), Some(&"d"));
        assert_eq!(table.get_cell(1, 2), None);
    }

    #[test]
    fn get_cell_should_respect_virtual_boundaries() {
        // Sets capacity to that of the 2D array provided
        let mut table = FixedColumnTable::from(vec![["a", "b"], ["c", "d"]]);
        assert_eq!(table.row_cnt(), 2);
        assert_eq!(table.col_cnt(), 2);

        // If we change the capacity to be smaller, get_cell should respect that
        table.set_row_capacity(1);
        table.set_column_capacity(1);
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.get_cell(0, 1).as_deref(), None);
        assert_eq!(table.get_cell(1, 0).as_deref(), None);
        assert_eq!(table.get_cell(1, 1).as_deref(), None);

        // Capacity changes don't actually overwrite anything
        table.set_row_capacity(2);
        table.set_column_capacity(2);
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.get_cell(0, 1).as_deref(), Some(&"b"));
        assert_eq!(table.get_cell(1, 0).as_deref(), Some(&"c"));
        assert_eq!(table.get_cell(1, 1).as_deref(), Some(&"d"));
    }

    #[test]
    fn get_mut_cell_should_return_mut_ref_to_cell_at_location() {
        let mut table = FixedColumnTable::from(vec![["a", "b"], ["c", "d"]]);
        *table.get_mut_cell(0, 0).unwrap() = "e";
        *table.get_mut_cell(0, 1).unwrap() = "f";
        *table.get_mut_cell(1, 0).unwrap() = "g";
        *table.get_mut_cell(1, 1).unwrap() = "h";
        assert_eq!(table.get_mut_cell(2, 0), None);

        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"e"));
        assert_eq!(table.get_cell(0, 1).as_deref(), Some(&"f"));
        assert_eq!(table.get_cell(1, 0).as_deref(), Some(&"g"));
        assert_eq!(table.get_cell(1, 1).as_deref(), Some(&"h"));
    }

    #[test]
    fn get_mut_cell_should_respect_virtual_boundaries() {
        let mut table = FixedColumnTable::from(vec![["a", "b"], ["c", "d"]]);
        assert_eq!(table.row_cnt(), 2);
        assert_eq!(table.col_cnt(), 2);

        // If we change the capacity to be smaller, get_mut_cell should respect that
        table.set_row_capacity(1);
        table.set_column_capacity(1);
        assert!(table.get_mut_cell(0, 0).is_some());
        assert!(table.get_mut_cell(0, 1).is_none());
        assert!(table.get_mut_cell(1, 0).is_none());
        assert!(table.get_mut_cell(1, 1).is_none());
    }

    #[test]
    fn insert_cell_should_return_previous_cell_and_overwrite_content() {
        let mut table: FixedColumnTable<usize, 3> = FixedColumnTable::new();

        assert_eq!(table.insert_cell(0, 0, 123), None);
        assert_eq!(table.insert_cell(0, 0, 999), Some(123));
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&999))
    }

    #[test]
    fn insert_cell_should_respect_actual_boundaries() {
        let mut table: FixedColumnTable<usize, 1> = FixedColumnTable::new();
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        // Can still create and expand within column
        assert_eq!(table.insert_cell(1, 0, 123), None);
        assert_eq!(table.get_cell(1, 0), Some(&123));

        // Creating anything outside of the single column will yield nothing
        assert_eq!(table.insert_cell(0, 1, 123), None);
        assert_eq!(table.insert_cell(1, 1, 123), None);
    }

    #[test]
    fn insert_cell_should_grow_virtual_boundaries_within_actual_limits() {
        let mut table: FixedColumnTable<usize, 3> = FixedColumnTable::new();
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        // Updating outside boundaries won't change anything
        table.insert_cell(3, 3, 123);
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        // Updating within limits should adjust accordingly
        table.insert_cell(2, 2, 123);
        assert_eq!(table.row_cnt(), 3);
        assert_eq!(table.col_cnt(), 3);
    }

    #[test]
    fn remove_cell_should_return_cell_that_is_removed() {
        let mut table = FixedColumnTable::from(vec![[1, 2], [3, 4]]);

        // NOTE: Because fixed table uses a default value when removing,
        //       we should see the default value of a number (0) be injected
        assert_eq!(table.remove_cell(0, 0), Some(1));
        assert_eq!(table.remove_cell(0, 0), Some(0));
    }

    #[test]
    fn remove_cell_should_respect_virtual_boundaries() {
        let mut table = FixedColumnTable::from(vec![[1, 2], [3, 4]]);
        table.set_row_capacity(0);
        table.set_column_capacity(0);

        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);
        assert_eq!(table.remove_cell(0, 0), None);
    }

    #[test]
    fn index_by_row_and_column_should_return_cell_ref() {
        let table = FixedColumnTable::from(vec![[1, 2, 3]]);
        assert_eq!(table[(0, 1)], 2);
    }

    #[test]
    #[should_panic]
    fn index_by_row_and_column_should_respect_virtual_boundaries() {
        let mut table = FixedColumnTable::from(vec![[1, 2, 3]]);
        table.set_row_capacity(0);
        table.set_column_capacity(0);

        // Will cause panic because of virtual boundary reached
        let _ = table[(0, 0)];
    }

    #[test]
    #[should_panic]
    fn index_by_row_and_column_should_panic_if_cell_not_found() {
        let table = FixedColumnTable::from(vec![[1, 2, 3]]);
        let _ = table[(1, 0)];
    }

    #[test]
    fn index_mut_by_row_and_column_should_return_mutable_cell() {
        let mut table = FixedColumnTable::from(vec![[1, 2, 3]]);
        table[(0, 1)] = 999;

        // Verify on the altered cell was changed
        assert_eq!(table[(0, 0)], 1);
        assert_eq!(table[(0, 1)], 999);
        assert_eq!(table[(0, 2)], 3);
    }

    #[test]
    #[should_panic]
    fn index_mut_by_row_and_column_should_respect_virtual_boundaries() {
        let mut table = FixedColumnTable::from(vec![[1, 2, 3]]);
        table.set_row_capacity(0);
        table.set_column_capacity(0);

        // Will cause panic because of virtual boundary reached
        table[(0, 0)] = 999;
    }

    #[test]
    #[should_panic]
    fn index_mut_by_row_and_column_should_panic_if_cell_not_found() {
        let mut table = FixedColumnTable::from(vec![[1, 2, 3]]);
        table[(1, 0)] = 999;
    }
}
