//! # memtable-macros
//!
//! Provides macros to derive tables.
//!
//! Check out full documentation at
//! [memtable](https://github.com/chipsenkbeil/memtable-rs).
#![forbid(unsafe_code)]

mod derive;
mod utils;

/// Derives an implementation of the **Table** trait from
/// [memtable](https://github.com/chipsenkbeil/memtable-rs)
///
/// Specifically, this produces a new struct to represent a table that can
/// store the target data as rows within itself. This will also produce a
/// unique data enum whose variants represent the different possible types
/// outlined by individual fields.
///
/// ### Table-wide Attributes
///
/// All table-level attributes use `#[table(...)]` as the starting point.
///
/// |Attribute Name|Usage                    |Description                                    |
/// |--------------|-------------------------|-----------------------------------------------|
/// |**mode**      |`mode = "dynamic"`       |Sets the mode to use when generating the table |
/// |**name**      |`name = "SomeOtherTable"`|Changes the name of the derived table          |
/// |**derive**    |`derive(Debug, ...)`     |Forwards derive attributes to the derived table|
/// |**skip_parts**|`skip_parts`             |Skips implementing `From` bidirectionally between the table and a tuple of its field types|
/// |**data**      |`data(...)`              |Specify attributes on a derived table's data   |
///
/// The mode attribute is a bit special in that it decides the underlying table
/// used to power the derived table. By default, `dynamic` is the mode used when
/// not specified.
///
/// ### Table Mode Settings
///
/// |Mode            |Usage                      |Description                               |
/// |----------------|---------------------------|------------------------------------------|
/// |**dynamic**     |`mode = "dynamic"`         |Produces a table that wraps `DynamicTable`|
/// |**fixed_column**|`mode = "fixed_column"`    |Produces a table that wraps `FixedColumnTable` where the total columns matches total fields|
/// |**fixed**       |`mode(fixed(rows = "..."))`|Produces a table that wraps `FixedTable` where the total columns matches total fields and the total rows is specified via the `rows` param|
///
/// ### Data-wide Attributes
///
/// All data-level attributes use `#[table(data(...))]` as the starting point.
///
/// |Attribute Name|Usage                   |Description                                    |
/// |--------------|------------------------|-----------------------------------------------|
/// |**name**      |`name = "SomeOtherData"`|Changes the name of the derived table data     |
/// |**derive**    |`derive(Debug, ...)`    |Forwards derive attributes to the derived data |
///
/// ### Column-specific Attributes
///
/// All column-specific attributes use `#[column(...)]` as the starting point.
///
/// |Attribute Name|Usage         |Description                                                     |
/// |--------------|--------------|----------------------------------------------------------------|
/// |**name**      |`name = "..."`|Changes the name of column when generating methods related to it|
/// |**indexed**   |`indexed`     |Flags the column as indexed for faster lookups at the cost of additional storage|
///
/// ### Examples
///
/// ```
/// # extern crate memtable_core as memtable;
/// use memtable_macros::Table;
///
/// // We define a table that stores users with a name and age
/// //
/// // 1. The table and its data derive Debug, which means we can print it
/// // 2. The mode has been changed to be a fixed column table underneath
/// // 3. The name column has been marked to be indexed
/// //
/// #[derive(Table)]
/// #[table(derive(Debug), data(derive(Debug)), mode = "fixed_column")]
/// struct User {
///     #[column(indexed)]
///     name: String,
///     age: u8,
/// }
///
/// let mut table = UserTable::new();
///
/// table.push_row(User {
///     name: "Fred Flintstone".to_string(),
///     age: 51,
/// });
///
/// table.push_row(User {
///     name: "Wilma Flintstone".to_string(),
///     age: 47,
/// });
///
/// // Retrieving data comes back as a tuple of all of the fields
/// let (name, age) = table.row(0).unwrap();
/// assert_eq!(name, "Fred Flintstone");
/// assert_eq!(*age, 51);
///
/// // We added a debug derive to the table, so we should be able to print it
/// println!("{:#?}", table);
/// ```
#[proc_macro_derive(Table, attributes(table, column))]
pub fn derive_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::do_derive(derive::do_derive_table)(input)
}
