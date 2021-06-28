use crate::Table;
use ::sled::{Config, Db, Tree};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, path::Path, sync::Mutex};

/// Total errors to keep around, dropping older ones after reaching limit
const ERROR_BUFFER_SIZE: usize = 10;

/// Represents a table that is replicated using sled
pub struct SledTable<D, T>
where
    D: Serialize + for<'de> Deserialize<'de>,
    T: Table<Data = D> + Default,
{
    db: Db,
    metadata: Tree,
    table: T,
    errors: Mutex<Vec<utils::Error>>,
}

impl<D, T> SledTable<D, T>
where
    D: Serialize + for<'de> Deserialize<'de>,
    T: Table<Data = D> + Default,
{
    /// Creates/reopens a sled database and populates an inmemory table with it
    pub fn new<P: AsRef<Path>>(path: P) -> utils::Result<Self> {
        let config = Config::default().path(path);
        Self::new_from_config(&config)
    }

    /// Creates/reopens a sled database using the provided config and
    /// populates an inmemory table with it
    pub fn new_from_config(config: &Config) -> utils::Result<Self> {
        let db = config.open()?;
        let metadata = db.open_tree(utils::metadata_name())?;
        let table = T::default();
        let errors = Mutex::new(Vec::new());

        let mut this = Self {
            db,
            metadata,
            table,
            errors,
        };

        this.reload()?;

        Ok(this)
    }

    /// Reloads the data in the table from sled
    pub fn reload(&mut self) -> utils::Result<()> {
        let (row_cnt, col_cnt) = utils::row_and_col_cnts(&self.metadata)?;
        let row_cnt = row_cnt.unwrap_or_default();
        let col_cnt = col_cnt.unwrap_or_default();

        self.table.set_row_capacity(row_cnt);
        self.table.set_column_capacity(col_cnt);

        for row in 0..row_cnt {
            for col in 0..col_cnt {
                let value = utils::load_cell(&self.db, row, col)?;
                if let Some(value) = value {
                    self.table.insert_cell(row, col, value);
                }
            }
        }
        Ok(())
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.lock().unwrap().is_empty()
    }

    pub fn clear_errors(&mut self) {
        self.errors.lock().unwrap().clear();
    }

    pub fn take_errors(&mut self) -> Vec<utils::Error> {
        self.errors.lock().unwrap().drain(..).collect()
    }

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
            utils::set_row_cnt(&self.metadata, self.table.row_cnt())?;
            utils::set_col_cnt(&self.metadata, self.table.col_cnt())?;

            for (pos, cell) in self.table.cells().zip_with_position() {
                let _ = utils::insert_cell(&self.db, pos.row, pos.col, cell)?;
            }
        }

        let cnt = self.db.flush()?;
        Ok(cnt)
    }
}

impl<D, T> Table for SledTable<D, T>
where
    D: Serialize + for<'de> Deserialize<'de>,
    T: Table<Data = D> + Default,
{
    type Data = D;

    fn row_cnt(&self) -> usize {
        self.table.row_cnt()
    }

    fn col_cnt(&self) -> usize {
        self.table.col_cnt()
    }

    fn get_cell(&self, row: usize, col: usize) -> Option<&Self::Data> {
        self.table.get_cell(row, col)
    }

    fn get_mut_cell(&mut self, row: usize, col: usize) -> Option<&mut Self::Data> {
        self.table.get_mut_cell(row, col)
    }

    fn insert_cell(&mut self, row: usize, col: usize, value: Self::Data) -> Option<Self::Data> {
        if let Err(x) = utils::insert_cell::<Self::Data>(&self.db, row, col, &value) {
            self.push_error(x);
        }

        let value = self.table.insert_cell(row, col, value);

        if let Err(x) =
            utils::set_row_and_col_cnts(&self.metadata, self.table.row_cnt(), self.table.col_cnt())
        {
            self.push_error(x);
        }

        value
    }

    fn remove_cell(&mut self, row: usize, col: usize) -> Option<Self::Data> {
        if let Err(x) = utils::remove_cell::<Self::Data>(&self.db, row, col) {
            self.push_error(x);
        }

        let value = self.table.remove_cell(row, col);

        if let Err(x) =
            utils::set_row_and_col_cnts(&self.metadata, self.table.row_cnt(), self.table.col_cnt())
        {
            self.push_error(x);
        }

        value
    }

    fn set_row_capacity(&mut self, capacity: usize) {
        if let Err(x) = utils::set_row_cnt(&self.metadata, capacity) {
            self.push_error(x);
        }

        self.table.set_row_capacity(capacity);
    }

    fn set_column_capacity(&mut self, capacity: usize) {
        if let Err(x) = utils::set_col_cnt(&self.metadata, capacity) {
            self.push_error(x);
        }

        self.table.set_column_capacity(capacity);
    }
}

impl<D, T> TryFrom<Db> for SledTable<D, T>
where
    D: Serialize + for<'de> Deserialize<'de>,
    T: Table<Data = D> + Default,
{
    type Error = utils::Error;

    /// Tries to create a database wrapper using sled, by loading a metadata
    /// collection and then attempting to populate the inmemory database
    /// using sled's current data
    fn try_from(db: Db) -> Result<Self, Self::Error> {
        let metadata = db.open_tree(utils::metadata_name())?;
        let table = T::default();
        let errors = Mutex::new(Vec::new());

        let mut this = Self {
            db,
            metadata,
            table,
            errors,
        };

        this.reload()?;

        Ok(this)
    }
}

mod utils {
    use ::sled::{
        transaction::{abort, TransactionError},
        Tree,
    };
    use serde::{Deserialize, Serialize};
    use std::{fmt, io};

    const METADATA_COLLECTION: &str = "memtable_metadata";
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

    pub const fn metadata_name() -> &'static str {
        METADATA_COLLECTION
    }

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

    pub fn set_row_cnt(tree: &Tree, row: usize) -> Result<()> {
        tree.insert(ROW_CNT_KEY, value_to_bytes(&row)?)?;
        Ok(())
    }

    pub fn set_col_cnt(tree: &Tree, col: usize) -> Result<()> {
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
        let mut buf = Vec::with_capacity((2 * usize::BITS / 8) as usize);
        buf.extend(&row.to_be_bytes());
        buf.extend(&col.to_be_bytes());

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DynamicTable;

    #[test]
    fn should_persist_across_creations() {
        // NOTE: Will be deleted once dropped; uses Arc<...> to track internally,
        //       so we can clone without issue
        let db = Config::default()
            .temporary(true)
            .open()
            .expect("Failed to create sled db");

        // First, load a clean table and populate it
        {
            let mut table = SledTable::<_, DynamicTable<usize>>::try_from(db.clone())
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
            let mut table =
                SledTable::<_, DynamicTable<usize>>::try_from(db).expect("Failed to load table");
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
