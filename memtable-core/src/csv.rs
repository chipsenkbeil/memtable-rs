use crate::Table;
use ::csv as csv_lib;
use std::{fs::File, io, path::Path};

impl Table<String> {
    /// Loads a table from some instance of the [`io::Read`] trait
    pub fn from_csv<R: io::Read>(reader: R) -> io::Result<Self> {
        let mut table = Table::new();

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

    /// Loads a table from a CSV str
    #[inline]
    pub fn from_csv_str(s: &str) -> io::Result<Self> {
        Self::from_csv(s.as_bytes())
    }

    /// Loads a table from a CSV file found at the given path
    #[inline]
    pub fn from_csv_file<P: AsRef<Path>>(p: P) -> io::Result<Self> {
        Self::from_csv(File::open(p)?)
    }
}

impl<T: AsRef<[u8]>> Table<T> {
    /// Writes a table to some instance of the [`io::Write`] trait
    pub fn to_csv<W: io::Write>(&self, writer: W) -> io::Result<()> {
        let mut wtr = csv_lib::WriterBuilder::new()
            .has_headers(false)
            .from_writer(writer);
        for row in self.rows() {
            wtr.write_record(row)?;
        }

        Ok(())
    }

    /// Write a table to a string
    #[inline]
    pub fn to_csv_str(&self) -> io::Result<String> {
        let mut buf = Vec::new();
        self.to_csv(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    /// Writes a table to a CSV file at the given path
    #[inline]
    pub fn to_csv_file<P: AsRef<Path>>(&self, p: P) -> io::Result<()> {
        self.to_csv(File::create(p)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn from_csv_str_should_convert_into_table() {
        let table = Table::from_csv_str("a,b,c\nd,e,f\n").unwrap();

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

        let table = Table::from_csv_file(file).unwrap();

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

        let mut table = Table::new();
        table.push_row(vec!["a", "b", "c"]);
        table.push_row(vec!["d", "e", "f"]);

        table.to_csv_file(file.path()).unwrap();

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!(buffer, "a,b,c\nd,e,f\n")
    }
}
