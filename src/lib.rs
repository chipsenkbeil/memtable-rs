//! # DynamicTable - Inmemory tables for use in Rust
//!
//! [![Build Status][build_img]][build_lnk]
//! [![Crates.io][crates_img]][crates_lnk]
//! [![Docs.rs][doc_img]][doc_lnk]
//!
//! ## Overview
//!
//! memtable provides a collection of table-oriented features for use inmemory.
//!
//! This crate acts as the aggregator of all subcrates such as `memtable-core`
//! and `memtable-macros` and should be the only crate imported when using
//! features from either.
//!
//! ## Installation
//!
//! ```toml
//! [dependencies]
//! memtable = "0.2"
//!
//! # Optionally, include features like `macros`
//! # memtable = { version = "0.2", features = ["macros"] }
//! ```
//!
//! ## Usage
//!
//! Most often, you will want to import the [`prelude`] to bring in relevant
//! traits and structs:
//!
//! ```rust
//! use memtable::prelude::*;
//!
//! // Create a 2x3 (row x column) table of integers
//! let mut table = FixedTable::from([
//!     [1, 2, 3],
//!     [4, 5, 6],
//! ]);
//!
//! // Examine one of the values, replace it, and examine again
//! assert_eq!(table[(1, 2)], 6);
//! table.insert_cell(1, 2, 999);
//! assert_eq!(table[(1, 2)], 999);
//! ```
//!
//! ## The Tables
//!
//! In the core library, you will find four primary tables:
//!
//! - [`DynamicTable`]: table with a dynamic capacity for rows & columns
//! - [`FixedTable`]: table with a fixed capacity for rows & columns
//! - [`FixedRowTable`]: table with a fixed capacity for rows & dynamic capacity for columns
//! - [`FixedColumnTable`]: table with a dynamic capacity for rows & fixed capacity for columns
//!
//! ## The Traits
//!
//! - [`Table`]: primary trait that exposes majority of common operations to
//!              perform on tables
//! - [`iter::CellIter`]: common trait that table iterators focused on
//!                       individual cells that enables zipping with a cell's
//!                       position and getting the current row & column of
//!                       the iterator
//!
//! ## The Extra Features
//!
//! Alongside the essentials, the library also provides several features that
//! provide extensions to the table arsenal:
//!
//! - **csv**: enables CSV support and
//!     - [`exts::csv::FromCsv`]: convert CSV into an inmemory table
//!     - [`exts::csv::ToCsv`]: convert an inmemory table to CSV
//! - **cell**: enables [`exts::cell::Cell2`] and more up to
//!             [`exts::cell::Cell26`], which represent generic enums that can
//!             be used as the data type for a table to enable multiple data
//!             types within a table (e.g. `DynamicTable<Cell2<String, bool>>`)
//! - **serde**: enables *serde* support on all table & cell implementations
//! - **sled**:  enables [`exts::sled::SledTable`], which provides persistent
//!              storage on top of other tables via the sled database
//! - **macros**: enables [`macro@Table`] macro to derive new struct that
//!               implements the [`Table`] trait to be able to store some
//!               struct into a dedicated, inmemory table
//!
//! ## The Macros
//!
//! Currently, there is a singular macro, [`macro@Table`], which is used to
//! derive a table to contain zero or more of a specific struct.
//!
//! ```rust
//! # #[cfg(not(feature = "macros"))]
//! # fn main() {}
//! # #[cfg(feature = "macros")]
//! # fn main() {
//! use memtable::Table;
//!
//! #[derive(Table)]
//! struct User {
//!     name: String,
//!     age: u8,
//! }
//!
//! // Derives a new struct, User{Table}, that can contain instances of User
//! // that are broken up into their individual fields
//! let mut table = UserTable::new();
//!
//! // Inserting is straightforward as a User is considered a singular row
//! table.push_row(User {
//!     name: "Fred Flintstone".to_string(),
//!     age: 51,
//! });
//!
//! // You can also pass in a tuple of the fields in order of declaration
//! table.push_row(("Wilma Flintstone".to_string(), 47));
//!
//! // Retrieval by row will provide the fields by ref as a tuple
//! let (name, age) = table.row(0).unwrap();
//! assert_eq!(name, "Fred Flintstone");
//! assert_eq!(*age, 51);
//!
//! // Tables of course provide a variety of other methods to inspect data
//! assert_eq!(
//!     table.name_column().collect::<Vec<&String>>(),
//!     vec!["Fred Flintstone", "Wilma Flintstone"],
//! );
//! # }
//! ```
//!
//! ## The License
//!
//! <sup>
//! Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
//! 2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
//! </sup>
//!
//! <br>
//!
//! <sub>
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in vimvar by you, as defined in the Apache-2.0 license, shall be
//! dual licensed as above, without any additional terms or conditions.
//! </sub>
//!
//! [build_img]: https://github.com/chipsenkbeil/memtable-rs/workflows/CI/badge.svg
//! [build_lnk]: https://github.com/chipsenkbeil/memtable-rs/actions
//! [crates_img]: https://img.shields.io/crates/v/memtable.svg
//! [crates_lnk]: https://crates.io/crates/memtable
//! [doc_img]: https://docs.rs/memtable/badge.svg
//! [doc_lnk]: https://docs.rs/memtable

pub use memtable_core::*;

#[cfg(feature = "macros")]
pub use memtable_macros::*;
