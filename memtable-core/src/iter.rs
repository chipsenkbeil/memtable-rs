use super::{Position, RefOrOwned, Table};
use std::marker::PhantomData;

/// Represents an iterator over some part of a table at the granularity
/// of individual cells within the table
pub trait CellIter<T>: Iterator<Item = T> + Sized {
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
    fn zip_with_position(self) -> ZipPosition<T, Self> {
        ZipPosition(self, PhantomData)
    }
}

/// Represents an iterator over some cell and its position
#[derive(Debug)]
pub struct ZipPosition<T, I: CellIter<T>>(I, PhantomData<T>);

impl<T, I: CellIter<T>> Iterator for ZipPosition<T, I> {
    type Item = (Position, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_with_pos()
    }
}

/// Represents an iterator over rows of a table
#[derive(Debug)]
pub struct Rows<'a, D, T: Table<Data = D>> {
    table: &'a T,
    idx: usize,
}

impl<'a, D, T: Table<Data = D>> Rows<'a, D, T> {
    /// Produces an iterator that will iterator through all rows from the
    /// beginning of the table
    pub fn new(table: &'a T) -> Self {
        Self { table, idx: 0 }
    }

    /// Produces an iterator that will return no rows
    pub fn empty(table: &'a T) -> Self {
        Self {
            table,
            idx: table.row_cnt(),
        }
    }
}

impl<'a, D, T: Table<Data = D>> Iterator for Rows<'a, D, T> {
    type Item = Row<'a, D, T>;

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

impl<'a, D, T: Table<Data = D>> ExactSizeIterator for Rows<'a, D, T> {}

/// Represents an iterator over cells within a row of a table
#[derive(Debug)]
pub struct Row<'a, D, T: Table<Data = D>> {
    table: &'a T,
    row: usize,
    col: usize,
}

impl<'a, D, T: Table<Data = D>> Row<'a, D, T> {
    /// Creates a new iterator over the cells in a row for the given table
    /// at the specified row
    pub fn new(table: &'a T, row: usize) -> Self {
        Self { table, row, col: 0 }
    }
}

impl<'a, D: 'a, T: Table<Data = D>> Iterator for Row<'a, D, T> {
    type Item = RefOrOwned<'a, D>;

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

impl<'a, D: 'a, T: Table<Data = D>> ExactSizeIterator for Row<'a, D, T> {}

impl<'a, D, T: Table<Data = D>> CellIter<RefOrOwned<'a, D>> for Row<'a, D, T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over cells within a row of a table
#[derive(Debug)]
pub struct IntoRow<D, T: Table<Data = D>> {
    table: T,
    row: usize,
    col: usize,
}

impl<D, T: Table<Data = D>> IntoRow<D, T> {
    pub fn new(table: T, row: usize) -> Self {
        Self { table, row, col: 0 }
    }
}

impl<'a, D, T: Table<Data = D>> From<&'a IntoRow<D, T>> for Row<'a, D, T> {
    fn from(it: &'a IntoRow<D, T>) -> Self {
        Self {
            table: &it.table,
            row: it.row,
            col: it.col,
        }
    }
}

impl<D, T: Table<Data = D>> Iterator for IntoRow<D, T> {
    type Item = D;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.remove_cell(self.row, self.col);
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

impl<D, T: Table<Data = D>> ExactSizeIterator for IntoRow<D, T> {}

impl<D, T: Table<Data = D>> CellIter<D> for IntoRow<D, T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over columns of a table
#[derive(Debug)]
pub struct Columns<'a, D, T: Table<Data = D>> {
    table: &'a T,
    idx: usize,
}

impl<'a, D, T: Table<Data = D>> Columns<'a, D, T> {
    /// Produces an iterator that will iterator through all columns from the
    /// beginning of the table
    pub fn new(table: &'a T) -> Self {
        Self { table, idx: 0 }
    }

    /// Produces an iterator that will return no columns
    pub fn empty(table: &'a T) -> Self {
        Self {
            table,
            idx: table.col_cnt(),
        }
    }
}

impl<'a, D, T: Table<Data = D>> Iterator for Columns<'a, D, T> {
    type Item = Column<'a, D, T>;

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

impl<'a, D, T: Table<Data = D>> ExactSizeIterator for Columns<'a, D, T> {}

/// Represents an iterator over cells within a column of a table
#[derive(Debug)]
pub struct Column<'a, D, T: Table<Data = D>> {
    table: &'a T,
    row: usize,
    col: usize,
}

impl<'a, D, T: Table<Data = D>> Column<'a, D, T> {
    /// Creates a new iterator over the cells in a column for the given table
    /// at the specified column
    pub fn new(table: &'a T, col: usize) -> Self {
        Self { table, row: 0, col }
    }
}

impl<'a, D: 'a, T: Table<Data = D>> Iterator for Column<'a, D, T> {
    type Item = RefOrOwned<'a, D>;

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

impl<'a, D: 'a, T: Table<Data = D>> ExactSizeIterator for Column<'a, D, T> {}

impl<'a, D, T: Table<Data = D>> CellIter<RefOrOwned<'a, D>> for Column<'a, D, T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over cells within a column of a table
#[derive(Debug)]
pub struct IntoColumn<D, T: Table<Data = D>> {
    table: T,
    row: usize,
    col: usize,
}

impl<D, T: Table<Data = D>> IntoColumn<D, T> {
    pub fn new(table: T, col: usize) -> Self {
        Self { table, row: 0, col }
    }
}

impl<'a, D, T: Table<Data = D>> From<&'a IntoColumn<D, T>> for Column<'a, D, T> {
    fn from(it: &'a IntoColumn<D, T>) -> Self {
        Self {
            table: &it.table,
            row: it.row,
            col: it.col,
        }
    }
}

impl<D, T: Table<Data = D>> Iterator for IntoColumn<D, T> {
    type Item = D;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.remove_cell(self.row, self.col);
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

impl<D, T: Table<Data = D>> ExactSizeIterator for IntoColumn<D, T> {}

impl<D, T: Table<Data = D>> CellIter<D> for IntoColumn<D, T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over cells within a table
#[derive(Debug)]
pub struct Cells<'a, D, T: Table<Data = D>> {
    table: &'a T,
    row: usize,
    col: usize,
}

impl<'a, D, T: Table<Data = D>> Cells<'a, D, T> {
    pub fn new(table: &'a T) -> Self {
        Self {
            table,
            row: 0,
            col: 0,
        }
    }
}

impl<'a, D: 'a, T: Table<Data = D>> Iterator for Cells<'a, D, T> {
    type Item = RefOrOwned<'a, D>;

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

impl<'a, D: 'a, T: Table<Data = D>> ExactSizeIterator for Cells<'a, D, T> {}

impl<'a, D, T: Table<Data = D>> CellIter<RefOrOwned<'a, D>> for Cells<'a, D, T> {
    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Represents an iterator over cells within a table
#[derive(Debug)]
pub struct IntoCells<D, T: Table<Data = D>> {
    table: T,
    row: usize,
    col: usize,
}

impl<D, T: Table<Data = D>> IntoCells<D, T> {
    pub fn new(table: T) -> Self {
        Self {
            table,
            row: 0,
            col: 0,
        }
    }
}

impl<'a, D, T: Table<Data = D>> From<&'a IntoCells<D, T>> for Cells<'a, D, T> {
    fn from(it: &'a IntoCells<D, T>) -> Self {
        Self {
            table: &it.table,
            row: it.row,
            col: it.col,
        }
    }
}

impl<D, T: Table<Data = D>> Iterator for IntoCells<D, T> {
    type Item = D;

    fn next(&mut self) -> Option<Self::Item> {
        let cell = self.table.remove_cell(self.row, self.col);
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

impl<D, T: Table<Data = D>> ExactSizeIterator for IntoCells<D, T> {}

impl<D, T: Table<Data = D>> CellIter<D> for IntoCells<D, T> {
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

    // NOTE: For simplicity, we use our one concrete implementor of the table
    //       trait as our test table
    type TestTable<T> = crate::MemDynamicTable<T>;

    fn make_hashmap<T>(items: Vec<(usize, usize, T)>) -> HashMap<Position, T> {
        items
            .into_iter()
            .map(|(row, col, x)| (Position { row, col }, x))
            .collect()
    }

    #[test]
    fn rows_next_should_return_next_row_if_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut rows = table.rows();
        assert!(rows.next().is_some());
    }

    #[test]
    fn rows_next_should_return_none_if_no_more_rows_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut rows = table.rows();
        rows.next();
        assert!(rows.next().is_none());
    }

    #[test]
    fn rows_size_hint_should_return_remaining_rows_as_both_bounds() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut rows = table.rows();
        assert_eq!(rows.size_hint(), (1, Some(1)));

        rows.next();
        assert_eq!(rows.size_hint(), (0, Some(0)));
    }

    #[test]
    fn row_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (1, 0, "c"),
            (1, 1, "d"),
            (2, 0, "e"),
            (2, 1, "f"),
        ]));

        assert_eq!(
            table.row(1).map(|x| *x).collect::<Vec<&str>>(),
            vec!["c", "d"]
        );
    }

    #[test]
    fn row_next_should_return_next_cell_if_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.row(0);
        assert!(row.next().is_some());
    }

    #[test]
    fn row_next_should_return_none_if_no_more_cells_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.row(0);
        row.next();
        assert!(row.next().is_none());
    }

    #[test]
    fn row_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.row(0);
        assert_eq!(row.size_hint(), (1, Some(1)));

        row.next();
        assert_eq!(row.size_hint(), (0, Some(0)));
    }

    #[test]
    fn into_row_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.into_row(0);
        assert!(row.next().is_some());
    }

    #[test]
    fn into_row_next_should_return_none_if_no_more_cells_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.into_row(0);
        row.next();
        assert!(row.next().is_none());
    }

    #[test]
    fn into_row_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut row = table.into_row(0);
        assert_eq!(row.size_hint(), (1, Some(1)));

        row.next();
        assert_eq!(row.size_hint(), (0, Some(0)));
    }

    #[test]
    fn columns_next_should_return_next_column_if_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut columns = table.columns();
        assert!(columns.next().is_some());
    }

    #[test]
    fn columns_next_should_return_none_if_no_more_columns_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut columns = table.columns();
        columns.next();
        assert!(columns.next().is_none());
    }

    #[test]
    fn columns_size_hint_should_return_remaining_columns_as_both_bounds() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut columns = table.columns();
        assert_eq!(columns.size_hint(), (1, Some(1)));

        columns.next();
        assert_eq!(columns.size_hint(), (0, Some(0)));
    }

    #[test]
    fn column_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        assert_eq!(
            table.column(1).map(|x| *x).collect::<Vec<&str>>(),
            vec!["b", "e"]
        );
    }

    #[test]
    fn column_next_should_return_next_cell_if_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.column(0);
        assert!(column.next().is_some());
    }

    #[test]
    fn column_next_should_return_none_if_no_more_cells_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.column(0);
        column.next();
        assert!(column.next().is_none());
    }

    #[test]
    fn column_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.column(0);
        assert_eq!(column.size_hint(), (1, Some(1)));

        column.next();
        assert_eq!(column.size_hint(), (0, Some(0)));
    }

    #[test]
    fn into_column_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.into_column(0);
        assert!(column.next().is_some());
    }

    #[test]
    fn into_column_next_should_return_none_if_no_more_cells_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.into_column(0);
        column.next();
        assert!(column.next().is_none());
    }

    #[test]
    fn into_column_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut column = table.into_column(0);
        assert_eq!(column.size_hint(), (1, Some(1)));

        column.next();
        assert_eq!(column.size_hint(), (0, Some(0)));
    }

    #[test]
    fn cells_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![
            (0, 0, "a"),
            (0, 1, "b"),
            (0, 2, "c"),
            (1, 0, "d"),
            (1, 1, "e"),
            (1, 2, "f"),
        ]));

        assert_eq!(
            table.cells().map(|x| *x).collect::<Vec<&str>>(),
            vec!["a", "b", "c", "d", "e", "f"]
        );
    }

    #[test]
    fn cells_next_should_return_next_cell_if_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.cells();
        assert!(cells.next().is_some());
    }

    #[test]
    fn cells_next_should_return_none_if_no_more_cells_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.cells();
        cells.next();
        assert!(cells.next().is_none());
    }

    #[test]
    fn cells_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.cells();
        assert_eq!(cells.size_hint(), (1, Some(1)));

        cells.next();
        assert_eq!(cells.size_hint(), (0, Some(0)));
    }

    #[test]
    fn into_cells_zip_with_position_should_map_iter_to_include_cell_position() {
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![
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
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.into_cells();
        assert!(cells.next().is_some());
    }

    #[test]
    fn into_cells_next_should_return_none_if_no_more_cells_available() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.into_cells();
        cells.next();
        assert!(cells.next().is_none());
    }

    #[test]
    fn into_cells_size_hint_should_return_remaining_cells_as_both_bounds() {
        let table = TestTable::from(make_hashmap(vec![(0, 0, "")]));

        let mut cells = table.into_cells();
        assert_eq!(cells.size_hint(), (1, Some(1)));

        cells.next();
        assert_eq!(cells.size_hint(), (0, Some(0)));
    }
}
