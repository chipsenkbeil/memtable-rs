use crate::{list::*, Capacity, Table};
use ::sled::Tree;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, sync::Mutex};

/// Total errors to keep around, dropping older ones after reaching limit
const ERROR_BUFFER_SIZE: usize = 10;

/// Represents a table that is replicated using a [`sled::Tree`]
#[derive(Debug)]
#[cfg_attr(feature = "docs", doc(cfg(sled)))]
pub struct SledTable<D, R, C, T>
where
    D: Serialize + for<'de> Deserialize<'de>,
    R: List<Item = D>,
    C: List<Item = D>,
    T: Table<Data = D, Row = R, Column = C>,
{
    tree: Tree,
    table: T,
    errors: Mutex<Vec<utils::Error>>,
}

impl<D, R, C, T> SledTable<D, R, C, T>
where
    D: Serialize + for<'de> Deserialize<'de>,
    R: List<Item = D>,
    C: List<Item = D>,
    T: Table<Data = D, Row = R, Column = C>,
{
    /// Creates a new sled table using the provided tree and factory function
    /// to create the inmemory table that takes in the current row and column
    /// capacities
    pub fn new(tree: Tree, new_table: impl FnOnce(usize, usize) -> T) -> utils::Result<Self> {
        // First, figure out our capacities if they have already been set
        // within the tree
        let (row_cnt, col_cnt) = utils::row_and_col_cnts(&tree)?;
        let row_cnt = row_cnt.unwrap_or_default();
        let col_cnt = col_cnt.unwrap_or_default();

        // Second, create our table instance and explicitly set the capacities
        let mut table = new_table(row_cnt, col_cnt);
        table.set_preferred_row_cnt(row_cnt);
        table.set_preferred_col_cnt(col_cnt);

        // Third, create our instance
        let mut this = Self {
            tree,
            table,
            errors: Mutex::new(Vec::new()),
        };

        // Fourth, load our data into the table (but don't pull capacities again)
        this.reload(false)?;

        // Fifth, return our new instance
        Ok(this)
    }

    /// Reloads the data in the table from sled, optionally refreshing the
    /// row and column capacities first
    pub fn reload(&mut self, refresh_capacities: bool) -> utils::Result<()> {
        let (row_cnt, col_cnt) = if refresh_capacities {
            let (row_cnt, col_cnt) = utils::row_and_col_cnts(&self.tree)?;
            let row_cnt = row_cnt.unwrap_or_default();
            let col_cnt = col_cnt.unwrap_or_default();

            self.table.set_preferred_row_cnt(row_cnt);
            self.table.set_preferred_col_cnt(col_cnt);
            (row_cnt, col_cnt)
        } else {
            (self.row_cnt(), self.col_cnt())
        };

        for row in 0..row_cnt {
            for col in 0..col_cnt {
                let value = utils::load_cell(&self.tree, row, col)?;
                if let Some(value) = value {
                    self.table.insert_cell(row, col, value);
                }
            }
        }
        Ok(())
    }

    /// Returns true if this table has uncleared errors
    pub fn has_errors(&self) -> bool {
        !self.errors.lock().unwrap().is_empty()
    }

    /// Removes errors in table without returning them
    pub fn clear_errors(&mut self) {
        self.errors.lock().unwrap().clear();
    }

    /// Removes errors in table and returns them
    pub fn take_errors(&mut self) -> Vec<utils::Error> {
        self.errors.lock().unwrap().drain(..).collect()
    }

    /// Adds a new error to the end of the queue, removing LRU errors until
    /// error buffer is at or under max capacity
    fn push_error(&mut self, e: impl Into<utils::Error>) {
        let mut errors = self.errors.lock().unwrap();
        errors.push(e.into());

        // Remove older errors past max buffer size
        if errors.len() > ERROR_BUFFER_SIZE {
            let extra = errors.len() - ERROR_BUFFER_SIZE;
            drop(errors.drain(0..extra));
        }
    }

    /// Flushes any changes to sled, optionally rewriting the entire table
    /// prior to flushing
    pub fn flush(&mut self, rewrite: bool) -> utils::Result<usize> {
        use crate::iter::CellIter;

        if rewrite {
            utils::set_preferred_row_cnt(&self.tree, self.table.row_cnt())?;
            utils::set_preferred_col_cnt(&self.tree, self.table.col_cnt())?;

            for (pos, cell) in self.table.cells().zip_with_position() {
                let _ = utils::insert_cell(&self.tree, pos.row, pos.col, cell)?;
            }
        }

        let cnt = self.tree.flush()?;
        Ok(cnt)
    }
}

impl<D, R, C, T> Table for SledTable<D, R, C, T>
where
    D: Serialize + for<'de> Deserialize<'de>,
    R: List<Item = D>,
    C: List<Item = D>,
    T: Table<Data = D, Row = R, Column = C>,
{
    type Data = D;
    type Row = R;
    type Column = C;

    fn max_row_capacity(&self) -> Capacity {
        self.table.max_row_capacity()
    }

    fn max_column_capacity(&self) -> Capacity {
        self.table.max_column_capacity()
    }

    fn row_cnt(&self) -> usize {
        self.table.row_cnt()
    }

    fn col_cnt(&self) -> usize {
        self.table.col_cnt()
    }

    fn cell(&self, row: usize, col: usize) -> Option<&Self::Data> {
        self.table.cell(row, col)
    }

    fn mut_cell(&mut self, row: usize, col: usize) -> Option<&mut Self::Data> {
        self.table.mut_cell(row, col)
    }

    /// Will insert the data into the cell, replicate it using the [`sled::Tree`],
    /// and update the metadata within the [`sled::Tree`] based on if the maximum
    /// row or column count has changed
    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data> {
        if let Err(x) = utils::insert_cell::<Self::Data>(&self.tree, row, col, &value) {
            self.push_error(x);
        }

        let value = self.table.insert_cell(row, col, value);

        if let Err(x) =
            utils::set_row_and_col_cnts(&self.tree, self.table.row_cnt(), self.table.col_cnt())
        {
            self.push_error(x);
        }

        value
    }

    /// Will remove the data from the cell, remove it from the [`sled::Tree`],
    /// and update the metadata within the [`sled::Tree`] based on if the maximum
    /// row or column count has changed
    fn remove_cell(&mut self, row: usize, col: usize) -> Option<Self::Data> {
        if let Err(x) = utils::remove_cell::<Self::Data>(&self.tree, row, col) {
            self.push_error(x);
        }

        let value = self.table.remove_cell(row, col);

        if let Err(x) =
            utils::set_row_and_col_cnts(&self.tree, self.table.row_cnt(), self.table.col_cnt())
        {
            self.push_error(x);
        }

        value
    }

    /// Will set the row capacity of the inner table and replicate the
    /// metadata in the [`sled::Tree`]
    fn set_preferred_row_cnt(&mut self, capacity: usize) {
        if let Err(x) = utils::set_preferred_row_cnt(&self.tree, capacity) {
            self.push_error(x);
        }

        self.table.set_preferred_row_cnt(capacity);
    }

    /// Will set the column capacity of the inner table and replicate the
    /// metadata in the [`sled::Tree`]
    fn set_preferred_col_cnt(&mut self, capacity: usize) {
        if let Err(x) = utils::set_preferred_col_cnt(&self.tree, capacity) {
            self.push_error(x);
        }

        self.table.set_preferred_col_cnt(capacity);
    }
}

impl<D, R, C, T> TryFrom<Tree> for SledTable<D, R, C, T>
where
    D: Serialize + for<'de> Deserialize<'de>,
    R: List<Item = D>,
    C: List<Item = D>,
    T: Table<Data = D, Row = R, Column = C> + Default,
{
    type Error = utils::Error;

    /// Tries to create a database wrapper using sled by mapping a tree's
    /// data to that of a table; will create a new instance of the inmemory
    /// table using its [`Default`] implementation
    fn try_from(tree: Tree) -> Result<Self, Self::Error> {
        Self::new(tree, |_, _| T::default())
    }
}

mod utils {
    use ::sled::{
        transaction::{abort, TransactionError},
        Tree,
    };
    use serde::{Deserialize, Serialize};
    use std::{fmt, io, mem};

    const ROW_CNT_KEY: &str = "row_cnt";
    const COL_CNT_KEY: &str = "col_cnt";

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug)]
    pub enum Error {
        FailedToSerialize(bincode::Error),
        FailedToDeserialize(bincode::Error),
        Io(io::Error),
        Sled(::sled::Error),
        MissingValue { key: String },
    }

    impl From<io::Error> for Error {
        fn from(x: io::Error) -> Self {
            Self::Io(x)
        }
    }

    impl<T: fmt::Display> From<TransactionError<T>> for Error {
        fn from(x: TransactionError<T>) -> Self {
            match x {
                TransactionError::Abort(x) => Self::Io(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    format!("{}", x),
                )),
                TransactionError::Storage(x) => Self::Sled(x),
            }
        }
    }

    impl From<sled::Error> for Error {
        fn from(x: sled::Error) -> Self {
            Self::Sled(x)
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self)
        }
    }

    impl std::error::Error for Error {}

    pub fn row_and_col_cnts(tree: &Tree) -> Result<(Option<usize>, Option<usize>)> {
        tree.transaction(|tx_db| {
            let row_cnt = tx_db
                .get(ROW_CNT_KEY)?
                .map(bytes_to_value)
                .transpose()
                .or_else(abort)?;
            let col_cnt = tx_db
                .get(COL_CNT_KEY)?
                .map(bytes_to_value)
                .transpose()
                .or_else(abort)?;

            Ok((row_cnt, col_cnt))
        })
        .map_err(Error::from)
    }

    pub fn set_row_and_col_cnts(tree: &Tree, row: usize, col: usize) -> Result<()> {
        tree.transaction(|tx_db| {
            tx_db.insert(ROW_CNT_KEY, value_to_bytes(&row).or_else(abort)?)?;
            tx_db.insert(COL_CNT_KEY, value_to_bytes(&col).or_else(abort)?)?;
            Ok(())
        })
        .map_err(Error::from)?;
        Ok(())
    }

    pub fn set_preferred_row_cnt(tree: &Tree, row: usize) -> Result<()> {
        tree.insert(ROW_CNT_KEY, value_to_bytes(&row)?)?;
        Ok(())
    }

    pub fn set_preferred_col_cnt(tree: &Tree, col: usize) -> Result<()> {
        tree.insert(COL_CNT_KEY, value_to_bytes(&col)?)?;
        Ok(())
    }

    pub fn insert_cell<T: Serialize + for<'de> Deserialize<'de>>(
        tree: &Tree,
        row: usize,
        col: usize,
        value: &T,
    ) -> Result<Option<T>> {
        swap_value(tree, make_cell_key(row, col), value)
    }

    pub fn load_cell<T: for<'de> Deserialize<'de>>(
        tree: &Tree,
        row: usize,
        col: usize,
    ) -> Result<Option<T>> {
        load_value(tree, make_cell_key(row, col))
    }

    pub fn remove_cell<T: for<'de> Deserialize<'de>>(
        tree: &Tree,
        row: usize,
        col: usize,
    ) -> Result<Option<T>> {
        remove_value(tree, make_cell_key(row, col))
    }

    pub fn load_value<T: for<'de> Deserialize<'de>>(
        tree: &Tree,
        key: impl AsRef<[u8]>,
    ) -> Result<Option<T>> {
        let value = tree.get(key)?.map(bytes_to_value).transpose()?;
        Ok(value)
    }

    pub fn swap_value<T: Serialize + for<'de> Deserialize<'de>>(
        tree: &Tree,
        key: impl AsRef<[u8]>,
        value: &T,
    ) -> Result<Option<T>> {
        let bytes = value_to_bytes(value)?;
        tree.insert(key, bytes)?.map(bytes_to_value).transpose()
    }

    pub fn remove_value<T: for<'de> Deserialize<'de>>(
        tree: &Tree,
        key: impl AsRef<[u8]>,
    ) -> Result<Option<T>> {
        tree.remove(key)?.map(bytes_to_value).transpose()
    }

    fn value_to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
        bincode::serialize(value).map_err(Error::FailedToSerialize)
    }

    fn bytes_to_value<T: for<'de> Deserialize<'de>>(bytes: impl AsRef<[u8]>) -> Result<T> {
        let value = bincode::deserialize(bytes.as_ref()).map_err(Error::FailedToDeserialize)?;
        Ok(value)
    }

    fn make_cell_key(row: usize, col: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity((2 * mem::size_of::<usize>()) as usize);
        buf.extend(&row.to_be_bytes());
        buf.extend(&col.to_be_bytes());

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DynamicTable;
    use sled::Config;

    #[test]
    fn should_persist_across_creations() {
        let db = Config::default()
            .temporary(true)
            .open()
            .expect("Failed to create sled db");

        // NOTE: Will be deleted once dropped; uses Arc<...> to track internally,
        //       so we can clone this without issue
        let tree = db
            .open_tree("test_table")
            .expect("Failed to create test_table tree");

        // First, load a clean table and populate it
        {
            let mut table = SledTable::<
                _,
                DynamicList<usize>,
                DynamicList<usize>,
                DynamicTable<usize>,
            >::try_from(tree.clone())
            .expect("Failed to load table");
            assert!(table.is_empty(), "Table populated unexpectedly");

            table.push_row(vec![1, 2, 3, 4]);
            table.push_row(vec![5, 6, 7, 8]);
            table.flush(false).expect("Failed to persist to disk");

            // There should not have been errors
            if table.has_errors() {
                panic!(
                    "[1] Encountered errors with table: {}",
                    table
                        .take_errors()
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                );
            }
        }

        // Second, reload the table, which sled should populate
        {
            let mut table = SledTable::<
                _,
                DynamicList<usize>,
                DynamicList<usize>,
                DynamicTable<usize>,
            >::try_from(tree)
            .expect("Failed to load table");
            assert!(!table.is_empty(), "Table not populated on second run");

            assert_eq!(table.pop_row().expect("Missing row 2"), vec![5, 6, 7, 8]);
            assert_eq!(table.pop_row().expect("Missing row 1"), vec![1, 2, 3, 4]);
            assert_eq!(table.pop_row(), None);

            // There still should not have been errors
            if table.has_errors() {
                panic!(
                    "[2] Encountered errors with table: {}",
                    table
                        .take_errors()
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("\n")
                );
            }
        }
    }
}
