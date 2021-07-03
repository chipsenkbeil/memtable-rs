use crate::{iter::*, list::*, utils, Capacity, Position, Table};
use core::{
    cmp,
    iter::FromIterator,
    mem,
    ops::{Index, IndexMut},
};

/// Represents an inmemory table containing rows & columns of some data `T`
/// with a fixed capacity across both rows and columns
#[cfg_attr(feature = "docs", doc(cfg(any(alloc, std))))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct FixedTable<T: Default, const ROW: usize, const COL: usize> {
    /// Internal allocation of our table's data
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
    cells: [[T; COL]; ROW],

    /// Represents a tracker for how many rows out of our total capacity
    /// have been used
    row_cnt: usize,

    /// Represents a tracker for how many columns out of our total capacity
    /// have been used
    col_cnt: usize,
}

impl<T: Default, const ROW: usize, const COL: usize> FixedTable<T, ROW, COL> {
    /// Creates a new, empty table
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes all cells contained within the table that are outside the
    /// current row & column capacity
    pub fn truncate(&mut self) {
        // Go through all cells NOT part of our virtual space and fill them
        // in with the default value
        for row in self.row_cnt..ROW {
            for col in self.col_cnt..COL {
                self.cells[row][col] = T::default();
            }
        }
    }

    /// Returns an iterator over the cells and their positions within the table
    pub fn iter(&self) -> ZipPosition<&T, Cells<T, FixedTable<T, ROW, COL>>> {
        self.into_iter()
    }
}

impl<T: Default, const ROW: usize, const COL: usize> Default for FixedTable<T, ROW, COL> {
    /// Creates a new table with maximum allocation of ROWxCOL, but with a
    /// virtual space (current occupancy) of 0x0
    fn default() -> Self {
        Self {
            cells: utils::default_table_array::<T, ROW, COL>(),
            row_cnt: 0,
            col_cnt: 0,
        }
    }
}

impl<T: Default, const ROW: usize, const COL: usize> Table for FixedTable<T, ROW, COL> {
    type Data = T;
    type Row = FixedList<Self::Data, COL>;
    type Column = FixedList<Self::Data, ROW>;

    fn max_row_capacity(&self) -> Capacity {
        Capacity::Limited(ROW)
    }

    fn max_column_capacity(&self) -> Capacity {
        Capacity::Limited(COL)
    }

    fn row_cnt(&self) -> usize {
        self.row_cnt
    }

    fn col_cnt(&self) -> usize {
        self.col_cnt
    }

    fn cell(&self, row: usize, col: usize) -> Option<&Self::Data> {
        // Limit access to the virtual space that has been assigned
        if row < self.row_cnt && col < self.col_cnt {
            Some(&self.cells[row][col])
        } else {
            None
        }
    }

    fn mut_cell(&mut self, row: usize, col: usize) -> Option<&mut Self::Data> {
        // Limit access to the virtual space that has been assigned
        if row < self.row_cnt && col < self.col_cnt {
            Some(&mut self.cells[row][col])
        } else {
            None
        }
    }

    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data> {
        // Allow inserting anywhere in the allocated space, not just virtual
        if row < ROW && col < COL {
            let mut did_grow = false;
            if row >= self.row_cnt {
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
        if row < self.row_cnt && col < self.col_cnt {
            Some(mem::take(&mut self.cells[row][col]))
        } else {
            None
        }
    }

    /// Will adjust the internal row count tracker to the specified capacity,
    /// capping at ROW.
    ///
    /// Note that this does **not** remove any cells from the table in their
    /// old positions. Instead, this updates the virtual space within the
    /// table that is made available for methods like [`Table::cell`].
    ///
    /// If you want to remove the cells that are no longer within capacity,
    /// call [`Self::truncate`], which will reset them to their default value.
    fn set_preferred_row_cnt(&mut self, cnt: usize) {
        self.row_cnt = cmp::min(cnt, ROW);
    }

    /// Will adjust the internal column count tracker to the specified capacity,
    /// capping at COL.
    ///
    /// Note that this does **not** remove any cells from the table in their
    /// old positions. Instead, this updates the virtual space within the
    /// table that is made available for methods like [`Table::cell`].
    ///
    /// If you want to remove the cells that are no longer within capacity,
    /// call [`Self::truncate`], which will reset them to their default value.
    fn set_preferred_col_cnt(&mut self, cnt: usize) {
        self.col_cnt = cmp::min(cnt, COL);
    }
}

impl<
        T: Default,
        U,
        const T_ROW: usize,
        const T_COL: usize,
        const U_ROW: usize,
        const U_COL: usize,
    > PartialEq<[[U; U_COL]; U_ROW]> for FixedTable<T, T_ROW, T_COL>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[[U; U_COL]; U_ROW]) -> bool {
        self.row_cnt == U_ROW
            && self.col_cnt == U_COL
            && self.cells[..U_ROW]
                .iter()
                .zip(other.iter())
                .all(|(r1, r2)| r1[..U_COL] == r2[..U_COL])
    }
}

impl<T: Default, const ROW: usize, const COL: usize> From<[[T; COL]; ROW]>
    for FixedTable<T, ROW, COL>
{
    /// Creates a new table with the provided cells as a starting point. As
    /// there is no way to tell how much of the 2D array is being used, the
    /// assumption is that the full array is occupied.
    ///
    /// If this is incorrect, adjust the virtual row and column counts with
    /// [`Table::set_row_capacity`] and [`Table::set_column_capacity`] respectively.
    fn from(cells: [[T; COL]; ROW]) -> Self {
        Self {
            cells,
            row_cnt: ROW,
            col_cnt: COL,
        }
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
        self.cell(row, col).expect("Row/Column index out of range")
    }
}

impl<T: Default, const ROW: usize, const COL: usize> IndexMut<(usize, usize)>
    for FixedTable<T, ROW, COL>
{
    /// Indexes into a table by a specific row and column, returning a mutable
    /// reference to the cell if it exists, otherwise panicking
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        self.mut_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_should_create_an_empty_table() {
        let table: FixedTable<usize, 3, 3> = FixedTable::new();
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);
    }

    #[test]
    fn row_cnt_should_be_adjustable_up_to_const_max() {
        let mut table: FixedTable<usize, 0, 0> = FixedTable::new();
        assert_eq!(table.row_cnt(), 0);
        table.set_preferred_row_cnt(1);
        assert_eq!(table.row_cnt(), 0);

        let mut table: FixedTable<usize, 4, 0> = FixedTable::new();
        assert_eq!(table.row_cnt(), 0);
        table.set_preferred_row_cnt(5);
        assert_eq!(table.row_cnt(), 4);
    }

    #[test]
    fn col_cnt_should_be_adjustable_up_to_const_max() {
        let mut table: FixedTable<usize, 0, 0> = FixedTable::new();
        assert_eq!(table.col_cnt(), 0);
        table.set_preferred_col_cnt(1);
        assert_eq!(table.col_cnt(), 0);

        let mut table: FixedTable<usize, 0, 4> = FixedTable::new();
        assert_eq!(table.col_cnt(), 0);
        table.set_preferred_col_cnt(5);
        assert_eq!(table.col_cnt(), 4);
    }

    #[test]
    fn cell_should_return_ref_to_cell_at_location() {
        // Sets capacity to that of the 2D array provided
        let table = FixedTable::from([["a", "b"], ["c", "d"]]);
        assert_eq!(table.cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.cell(0, 1).as_deref(), Some(&"b"));
        assert_eq!(table.cell(1, 0).as_deref(), Some(&"c"));
        assert_eq!(table.cell(1, 1).as_deref(), Some(&"d"));
        assert_eq!(table.cell(1, 2), None);
    }

    #[test]
    fn cell_should_respect_virtual_boundaries() {
        // Sets capacity to that of the 2D array provided
        let mut table = FixedTable::from([["a", "b"], ["c", "d"]]);
        assert_eq!(table.row_cnt(), 2);
        assert_eq!(table.col_cnt(), 2);

        // If we change the capacity to be smaller, cell should respect that
        table.set_preferred_row_cnt(1);
        table.set_preferred_col_cnt(1);
        assert_eq!(table.cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.cell(0, 1).as_deref(), None);
        assert_eq!(table.cell(1, 0).as_deref(), None);
        assert_eq!(table.cell(1, 1).as_deref(), None);

        // Capacity changes don't actually overwrite anything
        table.set_preferred_row_cnt(2);
        table.set_preferred_col_cnt(2);
        assert_eq!(table.cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.cell(0, 1).as_deref(), Some(&"b"));
        assert_eq!(table.cell(1, 0).as_deref(), Some(&"c"));
        assert_eq!(table.cell(1, 1).as_deref(), Some(&"d"));
    }

    #[test]
    fn mut_cell_should_return_mut_ref_to_cell_at_location() {
        let mut table = FixedTable::from([["a", "b"], ["c", "d"]]);
        *table.mut_cell(0, 0).unwrap() = "e";
        *table.mut_cell(0, 1).unwrap() = "f";
        *table.mut_cell(1, 0).unwrap() = "g";
        *table.mut_cell(1, 1).unwrap() = "h";
        assert_eq!(table.mut_cell(2, 0), None);

        assert_eq!(table.cell(0, 0).as_deref(), Some(&"e"));
        assert_eq!(table.cell(0, 1).as_deref(), Some(&"f"));
        assert_eq!(table.cell(1, 0).as_deref(), Some(&"g"));
        assert_eq!(table.cell(1, 1).as_deref(), Some(&"h"));
    }

    #[test]
    fn mut_cell_should_respect_virtual_boundaries() {
        let mut table = FixedTable::from([["a", "b"], ["c", "d"]]);
        assert_eq!(table.row_cnt(), 2);
        assert_eq!(table.col_cnt(), 2);

        // If we change the capacity to be smaller, mut_cell should respect that
        table.set_preferred_row_cnt(1);
        table.set_preferred_col_cnt(1);
        assert!(table.mut_cell(0, 0).is_some());
        assert!(table.mut_cell(0, 1).is_none());
        assert!(table.mut_cell(1, 0).is_none());
        assert!(table.mut_cell(1, 1).is_none());
    }

    #[test]
    fn insert_cell_should_return_previous_cell_and_overwrite_content() {
        let mut table: FixedTable<usize, 3, 3> = FixedTable::new();

        assert_eq!(table.insert_cell(0, 0, 123), None);
        assert_eq!(table.insert_cell(0, 0, 999), Some(123));
        assert_eq!(table.cell(0, 0).as_deref(), Some(&999))
    }

    #[test]
    fn insert_cell_should_respect_actual_boundaries() {
        let mut table: FixedTable<usize, 1, 1> = FixedTable::new();
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        assert_eq!(table.insert_cell(1, 0, 123), None);
        assert_eq!(table.insert_cell(0, 1, 123), None);
        assert_eq!(table.insert_cell(1, 1, 123), None);
    }

    #[test]
    fn insert_cell_should_grow_virtual_boundaries_within_actual_limits() {
        let mut table: FixedTable<usize, 3, 3> = FixedTable::new();
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
        let mut table = FixedTable::from([[1, 2], [3, 4]]);

        // NOTE: Because fixed table uses a default value when removing,
        //       we should see the default value of a number (0) be injected
        assert_eq!(table.remove_cell(0, 0), Some(1));
        assert_eq!(table.remove_cell(0, 0), Some(0));
    }

    #[test]
    fn remove_cell_should_respect_virtual_boundaries() {
        let mut table = FixedTable::from([[1, 2], [3, 4]]);
        table.set_preferred_row_cnt(0);
        table.set_preferred_col_cnt(0);

        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);
        assert_eq!(table.remove_cell(0, 0), None);
    }

    #[test]
    fn index_by_row_and_column_should_return_cell_ref() {
        let table = FixedTable::from([[1, 2, 3]]);
        assert_eq!(table[(0, 1)], 2);
    }

    #[test]
    #[should_panic]
    fn index_by_row_and_column_should_respect_virtual_boundaries() {
        let mut table = FixedTable::from([[1, 2, 3]]);
        table.set_preferred_row_cnt(0);
        table.set_preferred_col_cnt(0);

        // Will cause panic because of virtual boundary reached
        let _ = table[(0, 0)];
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
    fn index_mut_by_row_and_column_should_respect_virtual_boundaries() {
        let mut table = FixedTable::from([[1, 2, 3]]);
        table.set_preferred_row_cnt(0);
        table.set_preferred_col_cnt(0);

        // Will cause panic because of virtual boundary reached
        table[(0, 0)] = 999;
    }

    #[test]
    #[should_panic]
    fn index_mut_by_row_and_column_should_panic_if_cell_not_found() {
        let mut table = FixedTable::from([[1, 2, 3]]);
        table[(1, 0)] = 999;
    }

    #[test]
    fn insert_row_should_append_if_comes_after_last_row_if_capacity_remaining() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        // Shrink our capacity from the starting maximum so we can add a row
        table.set_preferred_row_cnt(2);

        table.insert_row(2, ["x", "y", "z"].iter().copied());

        assert_eq!(table, [["a", "b", "c"], ["d", "e", "f"], ["x", "y", "z"]]);
    }

    #[test]
    fn insert_row_at_end_should_do_nothing_if_no_capacity_remaining() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        table.insert_row(3, ["x", "y", "z"].iter().copied());

        assert_eq!(table, [["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);
    }

    #[test]
    fn insert_row_should_shift_down_all_rows_on_or_after_specified_row() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        table.insert_row(1, ["x", "y", "z"].iter().copied());

        assert_eq!(table, [["a", "b", "c"], ["x", "y", "z"], ["d", "e", "f"]]);
    }

    #[test]
    fn insert_row_should_support_insertion_at_front() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        table.insert_row(0, ["x", "y", "z"].iter().copied());

        assert_eq!(table, [["x", "y", "z"], ["a", "b", "c"], ["d", "e", "f"]]);
    }

    #[test]
    fn push_row_should_insert_at_end_if_capacity_remaining() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"]]);

        // Shrink our capacity from the starting maximum so we can add a row
        table.set_preferred_row_cnt(1);

        table.push_row(["g", "h", "i"].iter().copied());

        assert_eq!(table, [["a", "b", "c"], ["g", "h", "i"]]);
    }

    #[test]
    fn push_row_should_do_nothing_if_no_capacity_remaining() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"]]);

        table.push_row(["g", "h", "i"].iter().copied());

        assert_eq!(table, [["a", "b", "c"], ["d", "e", "f"]]);
    }

    #[test]
    fn insert_column_should_append_if_comes_after_last_column_if_capacity_remaining() {
        let mut table = FixedTable::from([["a", "b", "c", "g"], ["d", "e", "f", "h"]]);

        // Shrink our capacity from the starting maximum so we can add a column
        table.set_preferred_col_cnt(3);

        table.insert_column(3, ["x", "y"].iter().copied());

        assert_eq!(table, [["a", "b", "c", "x"], ["d", "e", "f", "y"]]);
    }

    #[test]
    fn insert_column_should_shift_right_all_columns_on_or_after_specified_column() {
        let mut table = FixedTable::from([["a", "b", "c", "g"], ["d", "e", "f", "h"]]);

        table.insert_column(1, ["x", "y"].iter().copied());

        assert_eq!(table, [["a", "x", "b", "c"], ["d", "y", "e", "f"]]);
    }

    #[test]
    fn insert_column_should_support_insertion_at_front() {
        let mut table = FixedTable::from([["a", "b", "c", "g"], ["d", "e", "f", "h"]]);

        table.insert_column(0, ["x", "y"].iter().copied());

        assert_eq!(table, [["x", "a", "b", "c"], ["y", "d", "e", "f"]]);
    }

    #[test]
    fn insert_column_at_end_should_do_nothing_if_no_capacity_remaining() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"]]);

        table.insert_column(3, ["g", "h"].iter().copied());

        assert_eq!(table, [["a", "b", "c"], ["d", "e", "f"]]);
    }

    #[test]
    fn push_column_should_insert_at_end_if_capacity_remaining() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"]]);

        // Shrink our capacity from the starting maximum so we can add a column
        table.set_preferred_col_cnt(2);

        table.push_column(["g", "h"].iter().copied());

        assert_eq!(table, [["a", "b", "g"], ["d", "e", "h"]]);
    }

    #[test]
    fn push_column_should_do_nothing_if_no_capacity_remaining() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"]]);

        table.push_column(["g", "h"].iter().copied());

        assert_eq!(table, [["a", "b", "c"], ["d", "e", "f"]]);
    }

    #[test]
    fn remove_row_should_return_list_representing_removed_row() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        assert_eq!(table.remove_row(1).unwrap(), ["d", "e", "f"]);
    }

    #[test]
    fn remove_row_should_shift_rows_after_up() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        table.remove_row(1);

        assert_eq!(table, [["a", "b", "c"], ["g", "h", "i"]]);
    }

    #[test]
    fn remove_row_should_support_removing_from_front() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        assert_eq!(table.remove_row(0).unwrap(), ["a", "b", "c"]);
        assert_eq!(table, [["d", "e", "f"], ["g", "h", "i"]]);
    }

    #[test]
    fn remove_row_should_return_none_if_row_missing() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        assert_eq!(table.remove_row(3), None);
        assert_eq!(table, [["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);
    }

    #[test]
    fn pop_row_should_remove_last_row() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        assert_eq!(table.pop_row().unwrap(), ["g", "h", "i"]);
        assert_eq!(table, [["a", "b", "c"], ["d", "e", "f"]]);
    }

    #[test]
    fn remove_column_should_return_iterator_over_removed_column() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        assert_eq!(table.remove_column(1).unwrap(), ["b", "e", "h"]);
    }

    #[test]
    fn remove_column_should_shift_columns_after_left() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        table.remove_column(1);

        assert_eq!(table, [["a", "c"], ["d", "f"], ["g", "i"]]);
    }

    #[test]
    fn remove_column_should_support_removing_from_front() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        assert_eq!(table.remove_column(0).unwrap(), ["a", "d", "g"]);

        assert_eq!(table, [["b", "c"], ["e", "f"], ["h", "i"]]);
    }

    #[test]
    fn remove_column_should_return_none_if_column_missing() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        assert_eq!(table.remove_column(3), None);

        assert_eq!(table, [["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);
    }

    #[test]
    fn pop_column_should_remove_last_column() {
        let mut table = FixedTable::from([["a", "b", "c"], ["d", "e", "f"], ["g", "h", "i"]]);

        assert_eq!(table.pop_column().unwrap(), ["c", "f", "i"]);

        assert_eq!(table, [["a", "b"], ["d", "e"], ["g", "h"]]);
    }
}
