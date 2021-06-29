// NOTE: Macros is looking for memtable, so we map our core crate since that's
//       actually what is used underneath
extern crate memtable_core as memtable;

// #[derive(::memtable_macros::Table)]
// struct MyTuple(u8, String);

/*
mod derive;
mod hygiene;

/// Runs all ui tests - note that all tests run through trybuild must be done
/// in one test method unless we manually run cargo test with a single thread
///
/// UI tests only run on nightly
///
/// https://github.com/dtolnay/trybuild/issues/58
/// https://github.com/dtolnay/trybuild/issues/6
#[rustversion::attr(not(nightly), ignore)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/
*.rs");
}

*/


struct MyTuple(u8, String);
#[automatically_derived]
struct MyTupleTable(::memtable::DynamicTable<MyTupleTableData>);
#[automatically_derived]
enum MyTupleTableData {
    _0(u8),
    _1(String),
}
#[automatically_derived]
impl MyTupleTableData {
    pub fn is__0(&self) -> ::std::primitive::bool {
        match self {
            Self::_0(_) => true,
            _ => false,
        }
    }
    pub fn as__0(&self) -> ::std::option::Option<&u8> {
        match self {
            Self::_0(x) => ::std::option::Option::Some(x),
            _ => ::std::option::Option::None,
        }
    }
    pub fn as_mut__0(&mut self) -> ::std::option::Option<&mut u8> {
        match self {
            Self::_0(x) => ::std::option::Option::Some(x),
            _ => ::std::option::Option::None,
        }
    }
    pub fn into__0(self) -> ::std::option::Option<u8> {
        match self {
            Self::_0(x) => ::std::option::Option::Some(x),
            _ => ::std::option::Option::None,
        }
    }
    pub fn is__1(&self) -> ::std::primitive::bool {
        match self {
            Self::_1(_) => true,
            _ => false,
        }
    }
    pub fn as__1(&self) -> ::std::option::Option<&String> {
        match self {
            Self::_1(x) => ::std::option::Option::Some(x),
            _ => ::std::option::Option::None,
        }
    }
    pub fn as_mut__1(&mut self) -> ::std::option::Option<&mut String> {
        match self {
            Self::_1(x) => ::std::option::Option::Some(x),
            _ => ::std::option::Option::None,
        }
    }
    pub fn into__1(self) -> ::std::option::Option<String> {
        match self {
            Self::_1(x) => ::std::option::Option::Some(x),
            _ => ::std::option::Option::None,
        }
    }
}
#[automatically_derived]
impl ::std::convert::AsRef<::memtable::DynamicTable<MyTupleTableData>> for MyTupleTable {
    fn as_ref(&self) -> &::memtable::DynamicTable<MyTupleTableData> {
        &self.0
    }
}
#[automatically_derived]
impl ::std::default::Default for MyTupleTable {
    fn default() -> Self {
        Self(<::memtable::DynamicTable<MyTupleTableData> as ::std::default::Default>::default())
    }
}
#[automatically_derived]
impl ::std::ops::Deref for MyTupleTable {
    type Target = ::memtable::DynamicTable<MyTupleTableData>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[automatically_derived]
impl ::std::convert::From<MyTupleTable> for ::memtable::DynamicTable<MyTupleTableData> {
    fn from(x: MyTupleTable) -> Self {
        x.0
    }
}
#[automatically_derived]
impl ::std::convert::TryFrom<::memtable::DynamicTable<MyTupleTableData>> for MyTupleTable {
    type Error = &'static ::std::primitive::str;
    fn try_from(
        table: ::memtable::DynamicTable<MyTupleTableData>,
    ) -> ::std::result::Result<Self, Self::Error> {
        for row in 0..::memtable::Table::row_cnt(&table) {
            let cell = ::memtable::Table::get_cell(&table, row, 0);
            if cell.is_none() {
                return ::std::result::Result::Err("Cell in column 0/_0 is missing");
            }
            if !cell.unwrap().is_0() {
                return ::std::result::Result::Err("Cell in column 0/_0 is not of type u8");
            }
            let cell = ::memtable::Table::get_cell(&table, row, 1);
            if cell.is_none() {
                return ::std::result::Result::Err("Cell in column 1/_1 is missing");
            }
            if !cell.unwrap().is_1() {
                return ::std::result::Result::Err("Cell in column 1/_1 is not of type String");
            }
        }
        ::std::result::Result::Ok(Self(table))
    }
}
#[automatically_derived]
impl ::std::convert::From<MyTuple> for (u8, String) {
    /// Convert from struct to tuple of fields
    fn from(x: MyTuple) -> (u8, String) {
        (x.0, x.1)
    }
}
#[automatically_derived]
impl ::std::convert::From<(u8, String)> for MyTuple {
    /// Convert from tuple of fields to struct
    fn from((0, 1): (u8, String)) -> Self {
        Self(0, 1)
    }
}
impl ::memtable::Table for MyTupleTable {
    type Data = MyTupleTableData;
    fn row_cnt(&self) -> ::std::primitive::usize {
        ::memtable::Table::row_cnt(&self.0)
    }
    fn col_cnt(&self) -> ::std::primitive::usize {
        ::memtable::Table::col_cnt(&self.0)
    }
    fn get_cell(
        &self,
        row: ::std::primitive::usize,
        col: ::std::primitive::usize,
    ) -> ::std::option::Option<&Self::Data> {
        ::memtable::Table::get_cell(&self.0, row, col)
    }
    fn get_mut_cell(
        &mut self,
        row: ::std::primitive::usize,
        col: ::std::primitive::usize,
    ) -> ::std::option::Option<&mut Self::Data> {
        ::memtable::Table::get_mut_cell(&mut self.0, row, col)
    }
    fn insert_cell(
        &mut self,
        row: ::std::primitive::usize,
        col: ::std::primitive::usize,
        value: Self::Data,
    ) -> ::std::option::Option<Self::Data> {
        ::memtable::Table::insert_cell(&mut self.0, row, col, value)
    }
    fn remove_cell(
        &mut self,
        row: ::std::primitive::usize,
        col: ::std::primitive::usize,
    ) -> ::std::option::Option<Self::Data> {
        ::memtable::Table::remove_cell(&mut self.0, row, col)
    }
    fn set_row_capacity(&mut self, capacity: ::std::primitive::usize) {
        ::memtable::Table::set_row_capacity(&mut self.0, capacity);
    }
    fn set_column_capacity(&mut self, capacity: ::std::primitive::usize) {
        ::memtable::Table::set_column_capacity(&mut self.0, capacity);
    }
}
#[automatically_derived]
impl MyTupleTable {
    pub fn new() -> Self {
        <Self as ::std::default::Default>::default()
    }
    /// Returns the numbers of the columns associated with this type of table
    pub fn column_names() -> &'static [&'static ::std::primitive::str] {
        &["0", "1"]
    }
    /// Retrieves a column by its name
    pub fn column_by_name(
        &self,
        name: &::std::primitive::str,
    ) -> ::std::option::Option<
        ::memtable::iter::Column<MyTupleTableData, ::memtable::DynamicTable<MyTupleTableData>>,
    > {
        match name {
            "0" => ::std::option::Option::Some(::memtable::Table::column(&self.0, 0)),
            "1" => ::std::option::Option::Some(::memtable::Table::column(&self.0, 1)),
            _ => ::std::option::Option::None,
        }
    }
    /// Converts into a column by its name
    pub fn into_column_by_name(
        self,
        name: &::std::primitive::str,
    ) -> ::std::option::Option<
        ::memtable::iter::IntoColumn<MyTupleTableData, ::memtable::DynamicTable<MyTupleTableData>>,
    > {
        match name {
            "0" => ::std::option::Option::Some(::memtable::Table::into_column(self.0, 0)),
            "1" => ::std::option::Option::Some(::memtable::Table::into_column(self.0, 1)),
            _ => ::std::option::Option::None,
        }
    }
    /// Iterates through each row of the table, returning a tuple of references
    /// to the individual fields
    pub fn rows(&self) -> impl ::std::iter::Iterator<Item = (&u8, &String)> {
        ::std::iter::Iterator::map(0..::memtable::Table::row_cnt(&self.0), move |idx| {
            self.row(idx).expect(
                "BUG: Typed row missing cell data! This should never happen! Please report this to
 https://github.com/chipsenkbeil/memtable-rs/issues",
            )
        })
    }
    /// Returns a tuple containing refs to each column's data within a row
    pub fn row(&self, row: ::std::primitive::usize) -> ::std::option::Option<(&u8, &String)> {
        todo!()
    }
    /// Inserts a new row into the table at the given position, shifting down
    /// all rows after it
    pub fn insert_row<__RowData: ::std::convert::Into<MyTuple>>(
        &mut self,
        row: ::std::primitive::usize,
        data: __RowData,
    ) {
    }
    /// Pushes a row to the end of the table
    pub fn push_row<__RowData: ::std::convert::Into<MyTuple>>(&mut self, data: __RowData) {
        self.insert_row(::memtable::Table::row_cnt(&self.0), data)
    }
    /// Removes the row at the specified position, shifting up all rows after it
    pub fn remove_row(&mut self, row: ::std::primitive::usize) -> ::std::option::Option<MyTuple> {
        :: memtable :: Table :: remove_row (& mut self . 0 , row) . and_then (| row | { let mut iter = :: std :: iter ::
IntoIterator :: into_iter (row) ; :: std :: option :: Option :: Some (MyTuple (:: std :: iter :: Iterator :: next (& mut
iter) . expect ("BUG: Typed row missing cell data! This should never happen! Please report this to https://github.com/chi
psenkbeil/memtable-rs/issues") . into__0 () . expect ("BUG: Typed row missing cell data! This should never happen! Please
 report this to https://github.com/chipsenkbeil/memtable-rs/issues") , :: std :: iter :: Iterator :: next (& mut iter) .
expect ("BUG: Typed row missing cell data! This should never happen! Please report this to https://github.com/chipsenkbei
l/memtable-rs/issues") . into__1 () . expect ("BUG: Typed row missing cell data! This should never happen! Please report
this to https://github.com/chipsenkbeil/memtable-rs/issues"))) })
    }
    /// Pops a row off the end of the table
    pub fn pop_row(&mut self) -> ::std::option::Option<MyTuple> {
        let max_rows = ::memtable::Table::row_cnt(&self.0);
        self.remove_row(if max_rows > 0 { max_rows - 1 } else { 0 })
    }
    pub fn get__0(&self, row: ::std::primitive::usize) -> ::std::option::Option<&u8> {
        ::memtable::Table::get_cell(&self.0, row, 0).and_then(MyTupleTableData::as__0)
    }
    pub fn get_mut__0(&mut self, row: ::std::primitive::usize) -> ::std::option::Option<&mut u8> {
        ::memtable::Table::get_mut_cell(&mut self.0, row, 0).and_then(MyTupleTableData::as_mut__0)
    }
    /// Swaps the current cell value with the provided one, doing nothing
    /// if there is no cell at the specified row for the explicit column
    pub fn replace__0<__Value: ::std::convert::Into<u8>>(
        &mut self,
        row: ::std::primitive::usize,
        value: __Value,
    ) -> ::std::option::Option<u8> {
        if row < ::memtable::Table::row_cnt(&self.0) {
            ::memtable::Table::insert_cell(&mut self.0, row, 0, MyTupleTableData::_0(value.into()))
                .and_then(MyTupleTableData::into__0)
        } else {
            ::std::option::Option::None
        }
    }
    pub fn _0_column(&self) -> impl ::std::iter::Iterator<Item = &u8> {
        let iter = ::memtable::Table::column(&self.0, 0);
        ::std::iter::Iterator::filter_map(iter, MyTupleTableData::as__0)
    }
    pub fn into__0_column(self) -> impl ::std::iter::Iterator<Item = u8> {
        let iter = ::memtable::Table::into_column(self.0, 0);
        ::std::iter::Iterator::filter_map(iter, MyTupleTableData::into__0)
    }
    pub fn get__1(&self, row: ::std::primitive::usize) -> ::std::option::Option<&String> {
        ::memtable::Table::get_cell(&self.0, row, 1).and_then(MyTupleTableData::as__1)
    }
    pub fn get_mut__1(
        &mut self,
        row: ::std::primitive::usize,
    ) -> ::std::option::Option<&mut String> {
        ::memtable::Table::get_mut_cell(&mut self.0, row, 1).and_then(MyTupleTableData::as_mut__1)
    }
    /// Swaps the current cell value with the provided one, doing nothing
    /// if there is no cell at the specified row for the explicit column
    pub fn replace__1<__Value: ::std::convert::Into<String>>(
        &mut self,
        row: ::std::primitive::usize,
        value: __Value,
    ) -> ::std::option::Option<String> {
        if row < ::memtable::Table::row_cnt(&self.0) {
            ::memtable::Table::insert_cell(&mut self.0, row, 1, MyTupleTableData::_1(value.into()))
                .and_then(MyTupleTableData::into__1)
        } else {
            ::std::option::Option::None
        }
    }
    pub fn _1_column(&self) -> impl ::std::iter::Iterator<Item = &String> {
        let iter = ::memtable::Table::column(&self.0, 1);
        ::std::iter::Iterator::filter_map(iter, MyTupleTableData::as__1)
    }
    pub fn into__1_column(self) -> impl ::std::iter::Iterator<Item = String> {
        let iter = ::memtable::Table::into_column(self.0, 1);
        ::std::iter::Iterator::filter_map(iter, MyTupleTableData::into__1)
    }
}
