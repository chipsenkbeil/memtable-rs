use crate::{iter::*, utils, Position, Table};
use std::{
    iter::FromIterator,
    mem,
    ops::{Index, IndexMut},
};

/// Represents an inmemory table containing rows & columns of some data `T`
/// with a fixed capacity across both rows and columns
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct FixedTable<T: Default, const ROW: usize, const COL: usize>(
    #[cfg_attr(
        feature = "serde-1",
        serde(
            bound(
                serialize = "T: serde::Serialize",
                deserialize = "T: serde::Deserialize<'de>"
            ),
            serialize_with = "utils::serialize_table_array",
            deserialize_with = "utils::deserialize_table_array"
        )
    )]
    [[T; COL]; ROW],
);

impl<T: Default, const ROW: usize, const COL: usize> FixedTable<T, ROW, COL> {
    /// Creates a new, empty table
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an iterator over the cells and their positions within the table
    pub fn iter(&self) -> ZipPosition<&T, Cells<T, FixedTable<T, ROW, COL>>> {
        self.into_iter()
    }
}

impl<T: Default, const ROW: usize, const COL: usize> Default for FixedTable<T, ROW, COL> {
    fn default() -> Self {
        Self(utils::default_table_array::<T, ROW, COL>())
    }
}

impl<T: Default, const ROW: usize, const COL: usize> Table for FixedTable<T, ROW, COL> {
    type Data = T;

    fn row_cnt(&self) -> usize {
        ROW
    }

    fn col_cnt(&self) -> usize {
        COL
    }

    fn get_cell(&self, row: usize, col: usize) -> Option<&Self::Data> {
        if row < ROW && col < COL {
            Some(&self.0[row][col])
        } else {
            None
        }
    }

    fn get_mut_cell(&mut self, row: usize, col: usize) -> Option<&mut Self::Data> {
        if row < ROW && col < COL {
            Some(&mut self.0[row][col])
        } else {
            None
        }
    }

    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data> {
        if row < ROW && col < COL {
            Some(mem::replace(&mut self.0[row][col], value))
        } else {
            None
        }
    }

    fn remove_cell(&mut self, row: usize, col: usize) -> Option<Self::Data> {
        self.insert_cell(row, col, T::default())
    }
}

impl<T: Default, const ROW: usize, const COL: usize> From<[[T; COL]; ROW]>
    for FixedTable<T, ROW, COL>
{
    fn from(cells: [[T; COL]; ROW]) -> Self {
        Self(cells)
    }
}

impl<'a, T: Default, const ROW: usize, const COL: usize> IntoIterator
    for &'a FixedTable<T, ROW, COL>
{
    type Item = (Position, &'a T);
    type IntoIter = ZipPosition<&'a T, Cells<'a, T, FixedTable<T, ROW, COL>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.cells().zip_with_position()
    }
}

impl<T: Default, const ROW: usize, const COL: usize> IntoIterator for FixedTable<T, ROW, COL> {
    type Item = (Position, T);
    type IntoIter = ZipPosition<T, IntoCells<T, FixedTable<T, ROW, COL>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.into_cells().zip_with_position()
    }
}

impl<T: Default, V: Into<T>, const ROW: usize, const COL: usize> FromIterator<(usize, usize, V)>
    for FixedTable<T, ROW, COL>
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

impl<T: Default, V: Into<T>, const ROW: usize, const COL: usize> FromIterator<(Position, V)>
    for FixedTable<T, ROW, COL>
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

impl<T: Default, const ROW: usize, const COL: usize> Index<(usize, usize)>
    for FixedTable<T, ROW, COL>
{
    type Output = T;

    /// Indexes into a table by a specific row and column, returning a
    /// reference to the cell if it exists, otherwise panicking
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        self.get_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

impl<T: Default, const ROW: usize, const COL: usize> IndexMut<(usize, usize)>
    for FixedTable<T, ROW, COL>
{
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
    fn row_cnt_should_match_fixed_row_size() {
        let table: FixedTable<usize, 0, 0> = FixedTable::new();
        assert_eq!(table.row_cnt(), 0);

        let table: FixedTable<usize, 4, 0> = FixedTable::new();
        assert_eq!(table.row_cnt(), 4);
    }

    #[test]
    fn col_cnt_should_match_fixed_col_size() {
        let table: FixedTable<usize, 0, 0> = FixedTable::new();
        assert_eq!(table.col_cnt(), 0);

        let table: FixedTable<usize, 0, 4> = FixedTable::new();
        assert_eq!(table.col_cnt(), 4);
    }

    #[test]
    fn get_cell_should_return_ref_to_cell_at_location() {
        let table = FixedTable::from([["a", "b"], ["c", "d"]]);
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.get_cell(0, 1).as_deref(), Some(&"b"));
        assert_eq!(table.get_cell(1, 0).as_deref(), Some(&"c"));
        assert_eq!(table.get_cell(1, 1).as_deref(), Some(&"d"));
        assert_eq!(table.get_cell(1, 2), None);
    }

    #[test]
    fn get_mut_cell_should_return_mut_ref_to_cell_at_location() {
        let mut table = FixedTable::from([["a", "b"], ["c", "d"]]);
        *table.get_mut_cell(0, 0).unwrap() = "e";
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"e"));
    }

    #[test]
    fn insert_cell_should_return_previous_cell_and_overwrite_content() {
        let mut table: FixedTable<usize, 3, 3> = FixedTable::new();

        assert_eq!(table.insert_cell(0, 0, 123), Some(usize::default()));
        assert_eq!(table.insert_cell(0, 0, 999), Some(123));
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&999))
    }

    #[test]
    fn remove_cell_should_return_cell_that_is_removed() {
        let mut table = FixedTable::from([[1, 2], [3, 4]]);

        // NOTE: Because fixed table uses a default value when removing,
        //       we should see the default value of a number (0) be injected
        assert_eq!(table.remove_cell(0, 0), Some(1));
        assert_eq!(table.remove_cell(0, 0), Some(0));
    }

    #[test]
    fn index_by_row_and_column_should_return_cell_ref() {
        let table = FixedTable::from([[1, 2, 3]]);
        assert_eq!(table[(0, 1)], 2);
    }

    #[test]
    #[should_panic]
    fn index_by_row_and_column_should_panic_if_cell_not_found() {
        let table = FixedTable::from([[1, 2, 3]]);
        let _ = table[(1, 0)];
    }

    #[test]
    fn index_mut_by_row_and_column_should_return_mutable_cell() {
        let mut table = FixedTable::from([[1, 2, 3]]);
        table[(0, 1)] = 999;

        // Verify on the altered cell was changed
        assert_eq!(table[(0, 0)], 1);
        assert_eq!(table[(0, 1)], 999);
        assert_eq!(table[(0, 2)], 3);
    }

    #[test]
    #[should_panic]
    fn index_mut_by_row_and_column_should_panic_if_cell_not_found() {
        let mut table = FixedTable::from([[1, 2, 3]]);
        table[(1, 0)] = 999;
    }
}
