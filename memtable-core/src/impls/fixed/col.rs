use super::utils;
use crate::{iter::*, Position, Table};
use std::{
    iter::FromIterator,
    mem,
    ops::{Index, IndexMut},
};

/// Represents an inmemory table containing rows & columns of some data `T`
/// with a fixed capacity across columns, but ability to grow dynamically with
/// rows
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct FixedColumnMemTable<T: Default, const COL: usize> {
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

    row_cnt: usize,
}

impl<T: Default, const COL: usize> FixedColumnMemTable<T, COL> {
    /// Creates a new, empty table
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes all cells contained within the table that are outside the
    /// current row capacity
    pub fn truncate(&mut self) {
        self.cells.truncate(self.row_cnt);
    }

    /// Shrinks the table's row capacity to fit where cells exist
    pub fn shrink_to_fit(&mut self) {
        self.row_cnt = self.cells.len();
    }

    /// Returns an iterator over the cells and their positions within the table
    pub fn iter(&self) -> ZipPosition<&T, Cells<T, FixedColumnMemTable<T, COL>>> {
        self.into_iter()
    }
}

impl<T: Default, const COL: usize> Table for FixedColumnMemTable<T, COL> {
    type Data = T;

    fn row_cnt(&self) -> usize {
        self.row_cnt
    }

    fn col_cnt(&self) -> usize {
        COL
    }

    fn get_cell(&self, row: usize, col: usize) -> Option<&Self::Data> {
        if row < self.row_cnt && col < COL {
            Some(&self.cells[row][col])
        } else {
            None
        }
    }

    fn get_mut_cell(&mut self, row: usize, col: usize) -> Option<&mut Self::Data> {
        if row < self.row_cnt && col < COL {
            Some(&mut self.cells[row][col])
        } else {
            None
        }
    }

    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data> {
        if col < COL {
            if row >= self.row_cnt {
                self.cells.resize_with(row + 1, utils::default_array);
                self.row_cnt = row + 1;
            }

            Some(mem::replace(&mut self.cells[row][col], value))
        } else {
            None
        }
    }

    fn remove_cell(&mut self, row: usize, col: usize) -> Option<T> {
        self.insert_cell(row, col, T::default())
    }

    /// Will adjust the internal row count tracker to the specified capacity
    ///
    /// Note that this does **not** remove any cells from the table in their
    /// old positions. To do that, call [`Self::truncate`].
    fn set_row_capacity(&mut self, capacity: usize) {
        self.row_cnt = capacity;
    }
}

impl<T: Default, const COL: usize> From<Vec<[T; COL]>> for FixedColumnMemTable<T, COL> {
    fn from(cells: Vec<[T; COL]>) -> Self {
        let row_cnt = cells.len();
        Self { cells, row_cnt }
    }
}

impl<'a, T: Default, const COL: usize> IntoIterator for &'a FixedColumnMemTable<T, COL> {
    type Item = (Position, &'a T);
    type IntoIter = ZipPosition<&'a T, Cells<'a, T, FixedColumnMemTable<T, COL>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.cells().zip_with_position()
    }
}

impl<T: Default, const COL: usize> IntoIterator for FixedColumnMemTable<T, COL> {
    type Item = (Position, T);
    type IntoIter = ZipPosition<T, IntoCells<T, FixedColumnMemTable<T, COL>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.into_cells().zip_with_position()
    }
}

impl<T: Default, V: Into<T>, const COL: usize> FromIterator<(usize, usize, V)>
    for FixedColumnMemTable<T, COL>
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
    for FixedColumnMemTable<T, COL>
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

impl<T: Default, const COL: usize> Index<(usize, usize)> for FixedColumnMemTable<T, COL> {
    type Output = T;

    /// Indexes into a table by a specific row and column, returning a
    /// reference to the cell if it exists, otherwise panicking
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        self.get_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

impl<T: Default, const COL: usize> IndexMut<(usize, usize)> for FixedColumnMemTable<T, COL> {
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
    fn row_cnt_should_be_dynamic() {
        let table: FixedColumnMemTable<usize, 0> = FixedColumnMemTable::new();
        assert_eq!(table.row_cnt(), 0);

        let mut table: FixedColumnMemTable<usize, 4> = FixedColumnMemTable::new();
        table.push_row([1, 2, 3, 4]);
        table.push_row([1, 2, 3, 4]);
        table.push_row([1, 2, 3, 4]);
        assert_eq!(table.row_cnt(), 3);
    }

    #[test]
    fn col_cnt_should_match_fixed_row_size() {
        let table: FixedColumnMemTable<usize, 0> = FixedColumnMemTable::new();
        assert_eq!(table.col_cnt(), 0);

        let table: FixedColumnMemTable<usize, 3> = FixedColumnMemTable::new();
        assert_eq!(table.col_cnt(), 3);
    }

    #[test]
    fn get_cell_should_return_ref_to_cell_at_location() {
        let table = FixedColumnMemTable::from(vec![["a", "b"], ["c", "d"]]);
        assert_eq!(table.get_cell(0, 0), Some(&"a"));
        assert_eq!(table.get_cell(0, 1), Some(&"b"));
        assert_eq!(table.get_cell(1, 0), Some(&"c"));
        assert_eq!(table.get_cell(1, 1), Some(&"d"));
        assert_eq!(table.get_cell(1, 2), None);
    }

    #[test]
    fn get_mut_cell_should_return_mut_ref_to_cell_at_location() {
        let mut table = FixedColumnMemTable::from(vec![["a", "b"], ["c", "d"]]);
        *table.get_mut_cell(0, 0).unwrap() = "e";
        assert_eq!(table.get_cell(0, 0), Some(&"e"));
    }

    #[test]
    fn insert_cell_should_return_previous_cell_and_overwrite_content() {
        let mut table: FixedColumnMemTable<usize, 3> = FixedColumnMemTable::new();

        assert_eq!(table.insert_cell(0, 0, 123), Some(usize::default()));
        assert_eq!(table.insert_cell(0, 0, 999), Some(123));
        assert_eq!(table.get_cell(0, 0), Some(&999))
    }

    #[test]
    fn remove_cell_should_return_cell_that_is_removed() {
        let mut table = FixedColumnMemTable::from(vec![[1, 2], [3, 4]]);

        // NOTE: Because fixed table uses a default value when removing,
        //       we should see the default value of a number (0) be injected
        assert_eq!(table.remove_cell(0, 0), Some(1));
        assert_eq!(table.remove_cell(0, 0), Some(0));
    }

    #[test]
    fn truncate_should_remove_cells_outside_of_row_capacity_count() {
        let mut table =
            FixedColumnMemTable::from(vec![["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        // Should do nothing if all cells are within capacities
        table.truncate();
        assert_eq!(
            table
                .iter()
                .map(|(pos, x)| (pos.row, pos.col, *x))
                .collect::<Vec<(usize, usize, &str)>>(),
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

        // Trucate from 3x3 to 2x3
        table.set_row_capacity(table.row_cnt() - 1);
        table.truncate();
        assert_eq!(
            table
                .iter()
                .map(|(pos, x)| (pos.row, pos.col, *x))
                .collect::<Vec<(usize, usize, &str)>>(),
            vec![
                (0, 0, "a"),
                (0, 1, "b"),
                (0, 2, "c"),
                (1, 0, "d"),
                (1, 1, "e"),
                (1, 2, "f")
            ]
        );
    }

    #[test]
    fn shrink_to_fit_should_adjust_row_count_based_on_cell_positions() {
        let mut table: FixedColumnMemTable<&'static str, 3> = FixedColumnMemTable::new();
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 3);

        table.cells.push(["a", "b", "c"]);
        table.cells.push(["d", "e", "f"]);
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 3);

        table.shrink_to_fit();
        assert_eq!(table.row_cnt(), 2);
        assert_eq!(table.col_cnt(), 3);
    }

    #[test]
    fn index_by_row_and_column_should_return_cell_ref() {
        let table = FixedColumnMemTable::from(vec![[1, 2, 3]]);
        assert_eq!(table[(0, 1)], 2);
    }

    #[test]
    #[should_panic]
    fn index_by_row_and_column_should_panic_if_cell_not_found() {
        let table = FixedColumnMemTable::from(vec![[1, 2, 3]]);
        let _ = table[(1, 0)];
    }

    #[test]
    fn index_mut_by_row_and_column_should_return_mutable_cell() {
        let mut table = FixedColumnMemTable::from(vec![[1, 2, 3]]);
        table[(0, 1)] = 999;

        // Verify on the altered cell was changed
        assert_eq!(table[(0, 0)], 1);
        assert_eq!(table[(0, 1)], 999);
        assert_eq!(table[(0, 2)], 3);
    }

    #[test]
    #[should_panic]
    fn index_mut_by_row_and_column_should_panic_if_cell_not_found() {
        let mut table = FixedColumnMemTable::from(vec![[1, 2, 3]]);
        table[(1, 0)] = 999;
    }
}
