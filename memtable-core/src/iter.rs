use super::{Position, Table};
use std::marker::PhantomData;

/// Represents an iterator over some part of a table at the granularity
/// of individual cells within the table
pub trait CellIter<T>: std::iter::Iterator<Item = T> + Sized {
    /// Returns the row of the next item returned by the iterator
    fn row(&self) -> usize;

    /// Returns the column of the next item returned by the iterator
    fn col(&self) -> usize;

    /// Consumes next item in iterator, returning it with the cell's position
    fn next_with_pos(&mut self) -> Option<(Position, T)> {
        let pos = Position {
            row: self.row(),
            col: self.col(),
        };
        self.next().map(move |x| (pos, x))
    }

    /// Zips up a cell iterator with the cell's position
    fn zip_with_position(self) -> ZipPosition<Self, T> {
        ZipPosition(self, PhantomData)
    }
}

/// Represents an iterator over some cell and its position
#[derive(Debug)]
pub struct ZipPosition<I: CellIter<T>, T>(I, PhantomData<T>);

impl<I: CellIter<T>, T> Iterator for ZipPosition<I, T> {
    type Item = (Position, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_with_pos()
    }
}

/// Represents an iterator over rows of a table
#[derive(Debug)]
pub struct Rows<'a, T> {
    table: &'a Table<T>,
    idx: usize,
}

impl<'a, T> Rows<'a, T> {
    /// Produces an iterator that will iterator through all rows from the
    /// beginning of the table
    pub fn new(table: &'a Table<T>) -> Self {
        Self { table, idx: 0 }
    }

    /// Produces an iterator that will return no rows
    pub fn empty(table: &'a Table<T>) -> Self {
        Self {
            table,
            idx: table.row_cnt(),
        }
    }
}

impl<'a, T> Iterator for Rows<'a, T> {
    type Item = Row<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.table.row_cnt() {
            let row = Row::new(self.table, self.idx);
            self.idx += 1;
            Some(row)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.row_cnt() - self.idx;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Rows<'a, T> {}

/// Represents an iterator over cells within a row of a table
#[derive(Debug)]
pub struct Row<'a, T> {
    table: &'a Table<T>,
    row: usize,
    col: usize,
}

impl<'a, T> Row<'a, T> {
    /// Creates a new iterator over the cells in a row for the given table
    /// at the specified row
    pub fn new(table: &'a Table<T>, row: usize) -> Self {
        Self { table, row, col: 0 }
    }
}

impl<'a, T> Iterator for Row<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.get_cell(self.row, self.col);
        if cell.is_some() {
            self.col += 1;
        }
        cell
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.col_cnt() - self.col;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Row<'a, T> {}

impl<'a, T> CellIter<&'a T> for Row<'a, T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over cells within a row of a table
#[derive(Debug)]
pub struct IntoRow<T> {
    table: Table<T>,
    row: usize,
    col: usize,
}

impl<T> IntoRow<T> {
    pub fn new(table: Table<T>, row: usize) -> Self {
        Self { table, row, col: 0 }
    }
}

impl<'a, T> From<&'a IntoRow<T>> for Row<'a, T> {
    fn from(it: &'a IntoRow<T>) -> Self {
        Self {
            table: &it.table,
            row: it.row,
            col: it.col,
        }
    }
}

impl<T> Iterator for IntoRow<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.cells.remove(&Position::new(self.row, self.col));
        if cell.is_some() {
            self.col += 1;
        }
        cell
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.col_cnt() - self.col;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for IntoRow<T> {}

impl<T> CellIter<T> for IntoRow<T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over columns of a table
#[derive(Debug)]
pub struct Columns<'a, T> {
    table: &'a Table<T>,
    idx: usize,
}

impl<'a, T> Columns<'a, T> {
    /// Produces an iterator that will iterator through all columns from the
    /// beginning of the table
    pub fn new(table: &'a Table<T>) -> Self {
        Self { table, idx: 0 }
    }

    /// Produces an iterator that will return no columns
    pub fn empty(table: &'a Table<T>) -> Self {
        Self {
            table,
            idx: table.col_cnt(),
        }
    }
}

impl<'a, T> Iterator for Columns<'a, T> {
    type Item = Column<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.table.col_cnt() {
            let col = Column::new(self.table, self.idx);
            self.idx += 1;
            Some(col)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.col_cnt() - self.idx;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Columns<'a, T> {}

/// Represents an iterator over cells within a column of a table
#[derive(Debug)]
pub struct Column<'a, T> {
    table: &'a Table<T>,
    row: usize,
    col: usize,
}

impl<'a, T> Column<'a, T> {
    /// Creates a new iterator over the cells in a column for the given table
    /// at the specified column
    pub fn new(table: &'a Table<T>, col: usize) -> Self {
        Self { table, row: 0, col }
    }
}

impl<'a, T> Iterator for Column<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.get_cell(self.row, self.col);
        if cell.is_some() {
            self.row += 1;
        }
        cell
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.row_cnt() - self.row;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Column<'a, T> {}

impl<'a, T> CellIter<&'a T> for Column<'a, T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over cells within a column of a table
#[derive(Debug)]
pub struct IntoColumn<T> {
    table: Table<T>,
    row: usize,
    col: usize,
}

impl<T> IntoColumn<T> {
    pub fn new(table: Table<T>, col: usize) -> Self {
        Self { table, row: 0, col }
    }
}

impl<'a, T> From<&'a IntoColumn<T>> for Column<'a, T> {
    fn from(it: &'a IntoColumn<T>) -> Self {
        Self {
            table: &it.table,
            row: it.row,
            col: it.col,
        }
    }
}

impl<T> Iterator for IntoColumn<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.cells.remove(&Position::new(self.row, self.col));
        if cell.is_some() {
            self.row += 1;
        }
        cell
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.table.row_cnt() - self.row;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for IntoColumn<T> {}

impl<T> CellIter<T> for IntoColumn<T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over cells within a table
#[derive(Debug)]
pub struct Cells<'a, T> {
    table: &'a Table<T>,
    row: usize,
    col: usize,
}

impl<'a, T> Cells<'a, T> {
    pub fn new(table: &'a Table<T>) -> Self {
        Self {
            table,
            row: 0,
            col: 0,
        }
    }
}

impl<'a, T> Iterator for Cells<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.get_cell(self.row, self.col);
        let col_cnt = self.table.col_cnt();
        let row_cnt = self.table.row_cnt();

        // If not yet reached end of row, advance column ptr
        if self.col + 1 < col_cnt {
            self.col += 1;

        // Else if not yet reached end of all rows, advance row ptr and
        // reset column ptr
        } else if self.row + 1 < row_cnt {
            self.row += 1;
            self.col = 0;

        // Otherwise, we have reached the end, so ensure we are done
        } else {
            self.row = row_cnt;
            self.col = col_cnt;
        }

        cell
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let consumed = (self.row * self.table.col_cnt()) + self.col;
        let total = self.table.len();
        let remaining = if total > consumed {
            total - consumed
        } else {
            0
        };
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Cells<'a, T> {}

impl<'a, T> CellIter<&'a T> for Cells<'a, T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over cells within a table
#[derive(Debug)]
pub struct IntoCells<T> {
    table: Table<T>,
    row: usize,
    col: usize,
}

impl<T> IntoCells<T> {
    pub fn new(table: Table<T>) -> Self {
        Self {
            table,
            row: 0,
            col: 0,
        }
    }
}

impl<'a, T> From<&'a IntoCells<T>> for Cells<'a, T> {
    fn from(it: &'a IntoCells<T>) -> Self {
        Self {
            table: &it.table,
            row: it.row,
            col: it.col,
        }
    }
}

impl<T> Iterator for IntoCells<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.cells.remove(&Position::new(self.row, self.col));
        let col_cnt = self.table.col_cnt();
        let row_cnt = self.table.row_cnt();

        // If not yet reached end of row, advance column ptr
        if self.col + 1 < col_cnt {
            self.col += 1;

        // Else if not yet reached end of all rows, advance row ptr and
        // reset column ptr
        } else if self.row + 1 < row_cnt {
            self.row += 1;
            self.col = 0;

        // Otherwise, we have reached the end, so ensure we are done
        } else {
            self.row = row_cnt;
            self.col = col_cnt;
        }

        cell
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let consumed = (self.row * self.table.col_cnt()) + self.col;
        let total = self.table.len();
        let remaining = if total > consumed {
            total - consumed
        } else {
            0
        };
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for IntoCells<T> {}

impl<T> CellIter<T> for IntoCells<T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_hashmap<T>(items: Vec<(usize, usize, T)>) -> HashMap<Position, T> {
        items
            .into_iter()
            .map(|(row, col, x)| (Position { row, col }, x))
            .collect()
    }

    #[test]
    fn rows_next_should_return_next_row_if_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut rows = table.rows();
        assert!(rows.next().is_some());
    }

    #[test]
    fn rows_next_should_return_none_if_no_more_rows_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut rows = table.rows();
        rows.next();
        assert!(rows.next().is_none());
    }

    #[test]
    fn rows_size_hint_should_return_remaining_rows_as_both_bounds() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut rows = table.rows();
        assert_eq!(rows.size_hint(), (1, Some(1)));

        rows.next();
        assert_eq!(rows.size_hint(), (0, Some(0)));
    }

    #[test]
    fn row_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
            (2, 0, "e"),
            (2, 1, "f"),
        ]));

        let mut rows = table.rows();

        let mut row_0 = rows.next().unwrap().zip_with_position();
        assert_eq!(row_0.next().unwrap().0, Position { row: 0, col: 0 });
        assert_eq!(row_0.next().unwrap().0, Position { row: 0, col: 1 });

        let mut row_1 = rows.next().unwrap().zip_with_position();
        assert_eq!(row_1.next().unwrap().0, Position { row: 1, col: 0 });
        assert_eq!(row_1.next().unwrap().0, Position { row: 1, col: 1 });

        let mut row_2 = rows.next().unwrap().zip_with_position();
        assert_eq!(row_2.next().unwrap().0, Position { row: 2, col: 0 });
        assert_eq!(row_2.next().unwrap().0, Position { row: 2, col: 1 });
    }

    #[test]
    fn row_should_iterator_through_appropriate_cells() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
            (2, 0, "e"),
            (2, 1, "f"),
        ]));

        assert_eq!(table.row(1).collect::<Vec<&&str>>(), vec![&"c", &"d"]);
    }

    #[test]
    fn row_next_should_return_next_cell_if_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.row(0);
        assert!(row.next().is_some());
    }

    #[test]
    fn row_next_should_return_none_if_no_more_cells_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.row(0);
        row.next();
        assert!(row.next().is_none());
    }

    #[test]
    fn row_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.row(0);
        assert_eq!(row.size_hint(), (1, Some(1)));

        row.next();
        assert_eq!(row.size_hint(), (0, Some(0)));
    }

    #[test]
    fn into_row_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
            (2, 0, "e"),
            (2, 1, "f"),
        ]));

        let mut row_0 = table.clone().into_row(0).zip_with_position();
        assert_eq!(row_0.next().unwrap().0, Position { row: 0, col: 0 });
        assert_eq!(row_0.next().unwrap().0, Position { row: 0, col: 1 });

        let mut row_1 = table.clone().into_row(1).zip_with_position();
        assert_eq!(row_1.next().unwrap().0, Position { row: 1, col: 0 });
        assert_eq!(row_1.next().unwrap().0, Position { row: 1, col: 1 });

        let mut row_2 = table.into_row(2).zip_with_position();
        assert_eq!(row_2.next().unwrap().0, Position { row: 2, col: 0 });
        assert_eq!(row_2.next().unwrap().0, Position { row: 2, col: 1 });
    }

    #[test]
    fn into_row_should_iterator_through_appropriate_cells() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
            (2, 0, "e"),
            (2, 1, "f"),
        ]));

        assert_eq!(
            table.into_row(1).collect::<Vec<&'static str>>(),
            vec!["c", "d"]
        );
    }

    #[test]
    fn into_row_next_should_return_next_cell_if_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.into_row(0);
        assert!(row.next().is_some());
    }

    #[test]
    fn into_row_next_should_return_none_if_no_more_cells_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.into_row(0);
        row.next();
        assert!(row.next().is_none());
    }

    #[test]
    fn into_row_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.into_row(0);
        assert_eq!(row.size_hint(), (1, Some(1)));

        row.next();
        assert_eq!(row.size_hint(), (0, Some(0)));
    }

    #[test]
    fn columns_next_should_return_next_column_if_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut columns = table.columns();
        assert!(columns.next().is_some());
    }

    #[test]
    fn columns_next_should_return_none_if_no_more_columns_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut columns = table.columns();
        columns.next();
        assert!(columns.next().is_none());
    }

    #[test]
    fn columns_size_hint_should_return_remaining_columns_as_both_bounds() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut columns = table.columns();
        assert_eq!(columns.size_hint(), (1, Some(1)));

        columns.next();
        assert_eq!(columns.size_hint(), (0, Some(0)));
    }

    #[test]
    fn column_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        let mut columns = table.columns();

        let mut column_0 = columns.next().unwrap().zip_with_position();
        assert_eq!(column_0.next().unwrap().0, Position { row: 0, col: 0 });
        assert_eq!(column_0.next().unwrap().0, Position { row: 1, col: 0 });

        let mut column_1 = columns.next().unwrap().zip_with_position();
        assert_eq!(column_1.next().unwrap().0, Position { row: 0, col: 1 });
        assert_eq!(column_1.next().unwrap().0, Position { row: 1, col: 1 });

        let mut column_2 = columns.next().unwrap().zip_with_position();
        assert_eq!(column_2.next().unwrap().0, Position { row: 0, col: 2 });
        assert_eq!(column_2.next().unwrap().0, Position { row: 1, col: 2 });
    }

    #[test]
    fn column_should_iterator_through_appropriate_cells() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        assert_eq!(table.column(1).collect::<Vec<&&str>>(), vec![&"b", &"e"]);
    }

    #[test]
    fn column_next_should_return_next_cell_if_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.column(0);
        assert!(column.next().is_some());
    }

    #[test]
    fn column_next_should_return_none_if_no_more_cells_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.column(0);
        column.next();
        assert!(column.next().is_none());
    }

    #[test]
    fn column_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.column(0);
        assert_eq!(column.size_hint(), (1, Some(1)));

        column.next();
        assert_eq!(column.size_hint(), (0, Some(0)));
    }

    #[test]
    fn into_column_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        let mut column_0 = table.clone().into_column(0).zip_with_position();
        assert_eq!(column_0.next().unwrap().0, Position { row: 0, col: 0 });
        assert_eq!(column_0.next().unwrap().0, Position { row: 1, col: 0 });

        let mut column_1 = table.clone().into_column(1).zip_with_position();
        assert_eq!(column_1.next().unwrap().0, Position { row: 0, col: 1 });
        assert_eq!(column_1.next().unwrap().0, Position { row: 1, col: 1 });

        let mut column_2 = table.into_column(2).zip_with_position();
        assert_eq!(column_2.next().unwrap().0, Position { row: 0, col: 2 });
        assert_eq!(column_2.next().unwrap().0, Position { row: 1, col: 2 });
    }

    #[test]
    fn into_column_should_iterator_through_appropriate_cells() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        assert_eq!(
            table.into_column(1).collect::<Vec<&'static str>>(),
            vec!["b", "e"]
        );
    }

    #[test]
    fn into_column_next_should_return_next_cell_if_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.into_column(0);
        assert!(column.next().is_some());
    }

    #[test]
    fn into_column_next_should_return_none_if_no_more_cells_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.into_column(0);
        column.next();
        assert!(column.next().is_none());
    }

    #[test]
    fn into_column_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.into_column(0);
        assert_eq!(column.size_hint(), (1, Some(1)));

        column.next();
        assert_eq!(column.size_hint(), (0, Some(0)));
    }

    #[test]
    fn cells_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        let mut cells = table.cells().zip_with_position();
        assert_eq!(cells.next().unwrap().0, Position { row: 0, col: 0 });
        assert_eq!(cells.next().unwrap().0, Position { row: 0, col: 1 });
        assert_eq!(cells.next().unwrap().0, Position { row: 0, col: 2 });
        assert_eq!(cells.next().unwrap().0, Position { row: 1, col: 0 });
        assert_eq!(cells.next().unwrap().0, Position { row: 1, col: 1 });
        assert_eq!(cells.next().unwrap().0, Position { row: 1, col: 2 });
    }

    #[test]
    fn cells_should_iterator_through_appropriate_cells() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        assert_eq!(
            table.cells().collect::<Vec<&&str>>(),
            vec![&"a", &"b", &"c", &"d", &"e", &"f"]
        );
    }

    #[test]
    fn cells_next_should_return_next_cell_if_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.cells();
        assert!(cells.next().is_some());
    }

    #[test]
    fn cells_next_should_return_none_if_no_more_cells_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.cells();
        cells.next();
        assert!(cells.next().is_none());
    }

    #[test]
    fn cells_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.cells();
        assert_eq!(cells.size_hint(), (1, Some(1)));

        cells.next();
        assert_eq!(cells.size_hint(), (0, Some(0)));
    }

    #[test]
    fn into_cells_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        let mut cells = table.into_cells().zip_with_position();
        assert_eq!(cells.next().unwrap().0, Position { row: 0, col: 0 });
        assert_eq!(cells.next().unwrap().0, Position { row: 0, col: 1 });
        assert_eq!(cells.next().unwrap().0, Position { row: 0, col: 2 });
        assert_eq!(cells.next().unwrap().0, Position { row: 1, col: 0 });
        assert_eq!(cells.next().unwrap().0, Position { row: 1, col: 1 });
        assert_eq!(cells.next().unwrap().0, Position { row: 1, col: 2 });
    }

    #[test]
    fn into_cells_should_iterator_through_all_cells() {
        let table = Table::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        assert_eq!(
            table.into_cells().collect::<Vec<&'static str>>(),
            vec!["a", "b", "c", "d", "e", "f"]
        );
    }

    #[test]
    fn into_cells_next_should_return_next_cell_if_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.into_cells();
        assert!(cells.next().is_some());
    }

    #[test]
    fn into_cells_next_should_return_none_if_no_more_cells_available() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.into_cells();
        cells.next();
        assert!(cells.next().is_none());
    }

    #[test]
    fn into_cells_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = Table::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.into_cells();
        assert_eq!(cells.size_hint(), (1, Some(1)));

        cells.next();
        assert_eq!(cells.size_hint(), (0, Some(0)));
    }
}