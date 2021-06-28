use crate::{iter::*, utils, MutRefOrOwned, Position, RefOrOwned, Table};
use std::{
    iter::FromIterator,
    mem,
    ops::{Index, IndexMut},
};

/// Represents an inmemory table containing rows & columns of some data `T`
/// with a fixed capacity across rows, but ability to grow dynamically with
/// columns
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct MemFixedRowTable<T: Default, const ROW: usize> {
    #[cfg_attr(
        feature = "serde-1",
        serde(
            bound(
                serialize = "T: serde::Serialize",
                deserialize = "T: serde::Deserialize<'de>"
            ),
            serialize_with = "utils::serialize_array",
            deserialize_with = "utils::deserialize_array"
        )
    )]
    cells: [Vec<T>; ROW],

    col_cnt: usize,
}

impl<T: Default, const ROW: usize> MemFixedRowTable<T, ROW> {
    /// Creates a new, empty table
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes all cells contained within the table that are outside the
    /// current column capacity
    pub fn truncate(&mut self) {
        let col_cnt = self.col_cnt;
        self.cells.iter_mut().for_each(|x| x.truncate(col_cnt));
    }

    /// Shrinks the table's column capacity to fit where cells exist
    pub fn shrink_to_fit(&mut self) {
        let max_col: usize = self.cells.iter().map(|x| x.len()).max().unwrap_or_default();
        self.col_cnt = max_col;
    }

    /// Returns an iterator over the cells and their positions within the table
    pub fn iter(&self) -> ZipPosition<RefOrOwned<'_, T>, Cells<T, MemFixedRowTable<T, ROW>>> {
        self.into_iter()
    }

    /// Internal method to get cell that supports directly returning a reference
    /// since inmemory tables have access in that manner
    fn _get_cell(&self, row: usize, col: usize) -> Option<&T> {
        if row < ROW && col < self.col_cnt {
            Some(&self.cells[row][col])
        } else {
            None
        }
    }

    /// Internal method to get cell that supports directly returning a mutable
    /// reference since inmemory tables have access in that manner
    fn _get_mut_cell(&mut self, row: usize, col: usize) -> Option<&mut T> {
        if row < ROW && col < self.col_cnt {
            Some(&mut self.cells[row][col])
        } else {
            None
        }
    }
}

impl<T: Default, const ROW: usize> Default for MemFixedRowTable<T, ROW> {
    fn default() -> Self {
        Self {
            cells: utils::default_array::<Vec<T>, ROW>(),
            col_cnt: 0,
        }
    }
}

impl<T: Default, const ROW: usize> Table for MemFixedRowTable<T, ROW> {
    type Data = T;

    fn row_cnt(&self) -> usize {
        ROW
    }

    fn col_cnt(&self) -> usize {
        self.col_cnt
    }

    fn get_cell(&self, row: usize, col: usize) -> Option<RefOrOwned<'_, Self::Data>> {
        self._get_cell(row, col).map(RefOrOwned::from)
    }

    fn get_mut_cell(&mut self, row: usize, col: usize) -> Option<MutRefOrOwned<'_, Self::Data>> {
        self._get_mut_cell(row, col).map(MutRefOrOwned::from)
    }

    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data> {
        if row < ROW {
            if col >= self.col_cnt {
                self.cells[row].resize_with(col + 1, Default::default);
                self.col_cnt = col + 1;
            }

            Some(mem::replace(&mut self.cells[row][col], value))
        } else {
            None
        }
    }

    fn remove_cell(&mut self, row: usize, col: usize) -> Option<T> {
        self.insert_cell(row, col, T::default())
    }

    /// Will adjust the internal column count tracker to the specified capacity
    ///
    /// Note that this does **not** remove any cells from the table in their
    /// old positions. To do that, call [`Self::truncate`].
    fn set_column_capacity(&mut self, capacity: usize) {
        self.col_cnt = capacity;
    }
}

impl<T: Default, const ROW: usize> From<[Vec<T>; ROW]> for MemFixedRowTable<T, ROW> {
    fn from(cells: [Vec<T>; ROW]) -> Self {
        let col_cnt = if ROW > 0 { cells[0].len() } else { 0 };
        Self { cells, col_cnt }
    }
}

impl<'a, T: Default, const ROW: usize> IntoIterator for &'a MemFixedRowTable<T, ROW> {
    type Item = (Position, RefOrOwned<'a, T>);
    type IntoIter = ZipPosition<RefOrOwned<'a, T>, Cells<'a, T, MemFixedRowTable<T, ROW>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.cells().zip_with_position()
    }
}

impl<T: Default, const ROW: usize> IntoIterator for MemFixedRowTable<T, ROW> {
    type Item = (Position, T);
    type IntoIter = ZipPosition<T, IntoCells<T, MemFixedRowTable<T, ROW>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.into_cells().zip_with_position()
    }
}

impl<T: Default, V: Into<T>, const ROW: usize> FromIterator<(usize, usize, V)>
    for MemFixedRowTable<T, ROW>
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

impl<T: Default, V: Into<T>, const ROW: usize> FromIterator<(Position, V)>
    for MemFixedRowTable<T, ROW>
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

impl<T: Default, const ROW: usize> Index<(usize, usize)> for MemFixedRowTable<T, ROW> {
    type Output = T;

    /// Indexes into a table by a specific row and column, returning a
    /// reference to the cell if it exists, otherwise panicking
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        self._get_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

impl<T: Default, const ROW: usize> IndexMut<(usize, usize)> for MemFixedRowTable<T, ROW> {
    /// Indexes into a table by a specific row and column, returning a mutable
    /// reference to the cell if it exists, otherwise panicking
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        self._get_mut_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_cnt_should_match_fixed_row_size() {
        let table: MemFixedRowTable<usize, 0> = MemFixedRowTable::new();
        assert_eq!(table.row_cnt(), 0);

        let table: MemFixedRowTable<usize, 4> = MemFixedRowTable::new();
        assert_eq!(table.row_cnt(), 4);
    }

    #[test]
    fn col_cnt_should_be_dynamic() {
        let table: MemFixedRowTable<usize, 0> = MemFixedRowTable::new();
        assert_eq!(table.col_cnt(), 0);

        let mut table: MemFixedRowTable<usize, 1> = MemFixedRowTable::new();
        table.push_column(vec![1, 2, 3]);
        table.push_column(vec![1, 2, 3]);
        table.push_column(vec![1, 2, 3]);

        assert_eq!(table.col_cnt(), 3);
    }

    #[test]
    fn get_cell_should_return_ref_to_cell_at_location() {
        let table = MemFixedRowTable::from([vec!["a", "b"], vec!["c", "d"]]);
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.get_cell(0, 1).as_deref(), Some(&"b"));
        assert_eq!(table.get_cell(1, 0).as_deref(), Some(&"c"));
        assert_eq!(table.get_cell(1, 1).as_deref(), Some(&"d"));
        assert_eq!(table.get_cell(1, 2), None);
    }

    #[test]
    fn get_mut_cell_should_return_mut_ref_to_cell_at_location() {
        let mut table = MemFixedRowTable::from([vec!["a", "b"], vec!["c", "d"]]);
        *table.get_mut_cell(0, 0).unwrap() = "e";
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"e"));
    }

    #[test]
    fn insert_cell_should_return_previous_cell_and_overwrite_content() {
        let mut table: MemFixedRowTable<usize, 3> = MemFixedRowTable::new();

        assert_eq!(table.insert_cell(0, 0, 123), Some(0));
        assert_eq!(table.insert_cell(0, 0, 999), Some(123));
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&999))
    }

    #[test]
    fn remove_cell_should_return_cell_that_is_removed() {
        let mut table = MemFixedRowTable::from([vec![1, 2], vec![3, 4]]);

        // NOTE: Because fixed table uses a default value when removing,
        //       we should see the default value of a number (0) be injected
        assert_eq!(table.remove_cell(0, 0), Some(1));
        assert_eq!(table.remove_cell(0, 0), Some(0));
    }

    #[test]
    fn truncate_should_remove_cells_outside_of_column_capacity_count() {
        let mut table = MemFixedRowTable::from([
            vec!["a", "b", "c"],
            vec!["d", "e", "f"],
            vec!["g", "h", "i"],
        ]);

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

        // Trucate from 3x3 to 3x2
        table.set_column_capacity(table.col_cnt() - 1);
        table.truncate();
        assert_eq!(
            table
                .iter()
                .map(|(pos, x)| (pos.row, pos.col, *x))
                .collect::<Vec<(usize, usize, &str)>>(),
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
    fn shrink_to_fit_should_adjust_column_count_based_on_cell_positions() {
        let mut table: MemFixedRowTable<&'static str, 3> = MemFixedRowTable::new();
        assert_eq!(table.row_cnt(), 3);
        assert_eq!(table.col_cnt(), 0);

        table.cells[0].extend(vec!["a", "b"]);
        table.cells[1].extend(vec!["d", "e", "f"]);
        table.cells[2].extend(vec!["g"]);
        assert_eq!(table.row_cnt(), 3);
        assert_eq!(table.col_cnt(), 0);

        table.shrink_to_fit();
        assert_eq!(table.row_cnt(), 3);
        assert_eq!(table.col_cnt(), 3);
    }

    #[test]
    fn index_by_row_and_column_should_return_cell_ref() {
        let table = MemFixedRowTable::from([vec![1, 2, 3]]);
        assert_eq!(table[(0, 1)], 2);
    }

    #[test]
    #[should_panic]
    fn index_by_row_and_column_should_panic_if_cell_not_found() {
        let table = MemFixedRowTable::from([vec![1, 2, 3]]);
        let _ = table[(1, 0)];
    }

    #[test]
    fn index_mut_by_row_and_column_should_return_mutable_cell() {
        let mut table = MemFixedRowTable::from([vec![1, 2, 3]]);
        table[(0, 1)] = 999;

        // Verify on the altered cell was changed
        assert_eq!(table[(0, 0)], 1);
        assert_eq!(table[(0, 1)], 999);
        assert_eq!(table[(0, 2)], 3);
    }

    #[test]
    #[should_panic]
    fn index_mut_by_row_and_column_should_panic_if_cell_not_found() {
        let mut table = MemFixedRowTable::from([vec![1, 2, 3]]);
        table[(1, 0)] = 999;
    }
}
