use crate::Table;
use ::csv as csv_lib;
use std::{fs::File, io, path::Path};

/// Represents ability to load data from a CSV
#[cfg_attr(feature = "docs", doc(cfg(csv)))]
pub trait FromCsv {
    type Output;

    /// Loads a table from some instance of the [`io::Read`] trait
    fn from_csv<R: io::Read>(reader: R) -> io::Result<Self::Output>;

    /// Loads a table from a CSV str
    #[inline]
    fn from_csv_str(s: &str) -> io::Result<Self::Output> {
        Self::from_csv(s.as_bytes())
    }

    /// Loads a table from a CSV file found at the given path
    #[inline]
    fn from_csv_file<P: AsRef<Path>>(p: P) -> io::Result<Self::Output> {
        Self::from_csv(File::open(p)?)
    }
}

impl<T: Table<Data = String>> FromCsv for T {
    type Output = T;

    fn from_csv<R: io::Read>(reader: R) -> io::Result<Self::Output> {
        let mut table = T::default();

        let mut rdr = csv_lib::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(reader);
        for (row, result) in rdr.records().enumerate() {
            let record = result?;
            for col in 0..record.len() {
                table.insert_cell(row, col, record[col].to_string());
            }
        }

        Ok(table)
    }
}

/// Represents ability to save data to a CSV
#[cfg_attr(feature = "docs", doc(cfg(csv)))]
pub trait ToCsv {
    /// Writes a table to some instance of the [`io::Write`] trait
    fn to_csv<W: io::Write>(&self, writer: W) -> io::Result<()>;

    /// Write a table to a string
    #[inline]
    fn to_csv_str(&self) -> io::Result<String> {
        let mut buf = Vec::new();
        self.to_csv(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    /// Writes a table to a CSV file at the given path
    #[inline]
    fn to_csv_file<P: AsRef<Path>>(&self, p: P) -> io::Result<()> {
        self.to_csv(File::create(p)?)
    }
}

impl<D: AsRef<[u8]>, T: Table<Data = D>> ToCsv for T {
    fn to_csv<W: io::Write>(&self, writer: W) -> io::Result<()> {
        let mut wtr = csv_lib::WriterBuilder::new()
            .has_headers(false)
            .from_writer(writer);
        for row in self.rows() {
            wtr.write_record(row)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    // NOTE: For simplicity, we use our one concrete implementor of the table
    //       trait as our test table
    type TestTable<T> = crate::MemDynamicTable<T>;

    #[test]
    fn from_csv_str_should_convert_into_table() {
        let table = TestTable::from_csv_str("a,b,c\nd,e,f\n").unwrap();

        assert_eq!(table.row_cnt(), 2);
        assert_eq!(table.col_cnt(), 3);
        assert_eq!(table[(0, 0)], "a");
        assert_eq!(table[(0, 1)], "b");
        assert_eq!(table[(0, 2)], "c");
        assert_eq!(table[(1, 0)], "d");
        assert_eq!(table[(1, 1)], "e");
        assert_eq!(table[(1, 2)], "f");
    }

    #[test]
    fn from_csv_file_should_convert_into_table() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(&mut file, "a,b,c\nd,e,f").unwrap();

        let table = TestTable::from_csv_file(file).unwrap();

        assert_eq!(table.row_cnt(), 2);
        assert_eq!(table.col_cnt(), 3);
        assert_eq!(table[(0, 0)], "a");
        assert_eq!(table[(0, 1)], "b");
        assert_eq!(table[(0, 2)], "c");
        assert_eq!(table[(1, 0)], "d");
        assert_eq!(table[(1, 1)], "e");
        assert_eq!(table[(1, 2)], "f");
    }

    #[test]
    fn to_csv_str_should_convert_into_csv() {
        let mut file = NamedTempFile::new().unwrap();

        let mut table = TestTable::new();
        table.push_row(vec!["a", "b", "c"]);
        table.push_row(vec!["d", "e", "f"]);

        table.to_csv_file(file.path()).unwrap();

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!(buffer, "a,b,c\nd,e,f\n")
    }
}
