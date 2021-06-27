#![forbid(unsafe_code)]
//! # memtable-macros
//!
//! Provides macros to derive tables.
//!
//! Check out full documentation at
//! [memtable](https://github.com/chipsenkbeil/memtable-rs).

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
/// ### Examples
///
/// ```
/// # extern crate memtable_core as memtable;
/// use memtable_macros::Table;
///
/// #[derive(Table)]
/// struct User {
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
/// ```
#[proc_macro_derive(Table, attributes(table))]
pub fn derive_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::do_derive(derive::do_derive_table)(input)
}
