use crate::{iter::*, Position, Table};
use std::{
    collections::HashMap,
    iter::FromIterator,
    ops::{Index, IndexMut},
};

/// Represents an inmemory table containing rows & columns of some data `T`,
/// capable of growing and shrinking in size dynamically
#[cfg_attr(feature = "serde-1", serde_with::serde_as)]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct MemDynamicTable<T> {
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

impl<T> MemDynamicTable<T> {
    /// Creates a new, empty table
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes all cells contained within the table that are outside the
    /// current row & column capacity
    pub fn truncate(&mut self) {
        let row_cnt = self.row_cnt;
        let col_cnt = self.col_cnt;
        self.cells
            .retain(|pos, _| pos.row < row_cnt && pos.col < col_cnt);
    }

    /// Shrinks the table's row & column capacity to fit where cells exist
    pub fn shrink_to_fit(&mut self) {
        let (max_row, max_col) = self.cells.keys().fold((0, 0), |acc, pos| {
            (
                std::cmp::max(acc.0, pos.row + 1),
                std::cmp::max(acc.1, pos.col + 1),
            )
        });

        self.row_cnt = max_row;
        self.col_cnt = max_col;
    }

    /// Returns an iterator over the cells and their positions within the table
    pub fn iter(&self) -> ZipPosition<&T, Cells<T, MemDynamicTable<T>>> {
        self.into_iter()
    }
}

impl<T> Default for MemDynamicTable<T> {
    fn default() -> Self {
        Self {
            cells: HashMap::new(),
            row_cnt: 0,
            col_cnt: 0,
        }
    }
}

impl<T> Table for MemDynamicTable<T> {
    type Data = T;

    fn row_cnt(&self) -> usize {
        self.row_cnt
    }

    fn col_cnt(&self) -> usize {
        self.col_cnt
    }

    fn get_cell(&self, row: usize, col: usize) -> Option<&Self::Data> {
        self.cells.get(&Position { row, col })
    }

    fn get_mut_cell(&mut self, row: usize, col: usize) -> Option<&mut Self::Data> {
        self.cells.get_mut(&Position { row, col })
    }

    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data> {
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

    fn remove_cell(&mut self, row: usize, col: usize) -> Option<T> {
        self.cells.remove(&Position { row, col })
    }

    /// Will adjust the internal row count tracker to the specified capacity
    ///
    /// Note that this does **not** remove any cells from the table in their
    /// old positions. To do that, call [`Self::truncate`].
    fn set_row_capacity(&mut self, capacity: usize) {
        self.row_cnt = capacity;
    }

    /// Will adjust the internal column count tracker to the specified capacity
    ///
    /// Note that this does **not** remove any cells from the table in their
    /// old positions. To do that, call [`Self::truncate`].
    fn set_column_capacity(&mut self, capacity: usize) {
        self.col_cnt = capacity;
    }
}

impl<'a, T> IntoIterator for &'a MemDynamicTable<T> {
    type Item = (Position, &'a T);
    type IntoIter = ZipPosition<&'a T, Cells<'a, T, MemDynamicTable<T>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.cells().zip_with_position()
    }
}

impl<T> IntoIterator for MemDynamicTable<T> {
    type Item = (Position, T);
    type IntoIter = ZipPosition<T, IntoCells<T, MemDynamicTable<T>>>;

    /// Converts into an iterator over the table's cells' positions and values
    fn into_iter(self) -> Self::IntoIter {
        self.into_cells().zip_with_position()
    }
}

impl<T, V: Into<T>> FromIterator<(usize, usize, V)> for MemDynamicTable<T> {
    /// Produces a table from the provided iterator of (row, col, value)
    fn from_iter<I: IntoIterator<Item = (usize, usize, V)>>(iter: I) -> Self {
        let cells: HashMap<Position, T> = iter
            .into_iter()
            .map(|(row, col, x)| (Position { row, col }, x.into()))
            .collect();
        Self::from(cells)
    }
}

impl<T, V: Into<T>> FromIterator<(Position, V)> for MemDynamicTable<T> {
    /// Produces a table from the provided iterator of (position, value)
    fn from_iter<I: IntoIterator<Item = (Position, V)>>(iter: I) -> Self {
        let cells: HashMap<Position, T> = iter.into_iter().map(|(p, x)| (p, x.into())).collect();
        Self::from(cells)
    }
}

impl<T> From<HashMap<Position, T>> for MemDynamicTable<T> {
    /// Creates a new table from the given hashmap of cells
    fn from(cells: HashMap<Position, T>) -> Self {
        let mut table = Self {
            cells,
            row_cnt: 0,
            col_cnt: 0,
        };

        // Shrink will calculate the proper row and column counts
        table.shrink_to_fit();

        table
    }
}

impl<T> Index<(usize, usize)> for MemDynamicTable<T> {
    type Output = T;

    /// Indexes into a table by a specific row and column, returning a
    /// reference to the cell if it exists, otherwise panicking
    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        self.get_cell(row, col)
            .expect("Row/Column index out of range")
    }
}

impl<T> IndexMut<(usize, usize)> for MemDynamicTable<T> {
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
        let table = MemDynamicTable::from(make_empty_hashmap::<usize>());
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        let table = MemDynamicTable::from(make_hashmap(vec![(3, 2, "some value")]));
        assert_eq!(table.row_cnt(), 4);
        assert_eq!(table.col_cnt(), 3);

        let table = MemDynamicTable::from(make_hashmap(vec![(3, 0, "value"), (0, 5, "value")]));
        assert_eq!(table.row_cnt(), 4);
        assert_eq!(table.col_cnt(), 6);
    }

    #[test]
    fn get_cell_should_return_ref_to_cell_at_location() {
        let table = MemDynamicTable::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
        ]));
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"a"));
        assert_eq!(table.get_cell(0, 1).as_deref(), Some(&"b"));
        assert_eq!(table.get_cell(1, 0).as_deref(), Some(&"c"));
        assert_eq!(table.get_cell(1, 1).as_deref(), Some(&"d"));
        assert_eq!(table.get_cell(1, 2), None);
    }

    #[test]
    fn get_mut_cell_should_return_mut_ref_to_cell_at_location() {
        let mut table = MemDynamicTable::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
        ]));
        *table.get_mut_cell(0, 0).unwrap() = "e";
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"e"));
    }

    #[test]
    fn insert_cell_should_extend_max_row_size_if_adding_beyond_max_row() {
        let mut table = MemDynamicTable::new();

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
        let mut table = MemDynamicTable::new();

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
        let mut table = MemDynamicTable::new();

        assert!(table.insert_cell(0, 0, "test").is_none());
        assert_eq!(table.insert_cell(0, 0, "other"), Some("test"));
        assert_eq!(table.get_cell(0, 0).as_deref(), Some(&"other"))
    }

    #[test]
    fn remove_cell_should_return_cell_that_is_removed() {
        let mut table: MemDynamicTable<&'static str> =
            vec![(0, 0, "a"), (1, 1, "b")].into_iter().collect();

        assert_eq!(table.remove_cell(0, 0), Some("a"));
        assert!(table.remove_cell(0, 0).is_none());
    }

    #[test]
    fn truncate_should_remove_cells_outside_of_row_and_column_capacity_counts() {
        let mut table = MemDynamicTable::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
            (2, 0, "g"),
            (2, 1, "h"),
            (2, 2, "i"),
        ]));

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

        // Trucate from 3x3 to 2x2
        table.set_row_capacity(table.row_cnt() - 1);
        table.set_column_capacity(table.col_cnt() - 1);
        table.truncate();
        assert_eq!(
            table
                .iter()
                .map(|(pos, x)| (pos.row, pos.col, *x))
                .collect::<Vec<(usize, usize, &str)>>(),
            vec![(0, 0, "a"), (0, 1, "b"), (1, 0, "d"), (1, 1, "e")]
        );
    }

    #[test]
    fn shrink_to_fit_should_adjust_row_and_column_counts_based_on_cell_positions() {
        let mut table: MemDynamicTable<&'static str> = MemDynamicTable::new();
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        table.cells.insert(Position { row: 0, col: 3 }, "a");
        table.cells.insert(Position { row: 5, col: 0 }, "b");
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        table.shrink_to_fit();
        assert_eq!(table.row_cnt(), 6);
        assert_eq!(table.col_cnt(), 4);
    }

    #[test]
    fn index_by_row_and_column_should_return_cell_ref() {
        let mut table = MemDynamicTable::new();
        table.push_row(vec![1, 2, 3]);

        assert_eq!(table[(0, 1)], 2);
    }

    #[test]
    #[should_panic]
    fn index_by_row_and_column_should_panic_if_cell_not_found() {
        let mut table = MemDynamicTable::new();
        table.push_row(vec![1, 2, 3]);

        let _ = table[(1, 0)];
    }

    #[test]
    fn index_mut_by_row_and_column_should_return_mutable_cell() {
        let mut table = MemDynamicTable::new();
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
        let mut table = MemDynamicTable::new();
        table.push_row(vec![1, 2, 3]);

        table[(1, 0)] = 999;
    }
}
