//! # The DynamicTable Prelude
//!
//! The memtable library comes with a variety of tools to help with building,
//! parsing, and transforming tables. While these could be brought in via a
//! mixture of `use memtable::*;` and other imports, this prelude serves as
//! the one-stop shop to import required traits, common structs, and more
//! without polluting the namespace with public modules exposed by this crate.
//!
//! # Prelude contents
//!
//! The current version of the prelude re-exports the following:
//!
//! * [`DynamicTable`] struct, which is the core table available from
//!   this crate that acts as a table that can grow and shrink dynamically
//! * [`FixedTable`] struct - available with Rust 1.51+ - provides a fixed-sized
//!   counterpart to [`DynamicTable`] where the table is pre-allocated internally
//!   using a 2D array
//! * [`FixedRowTable`] struct, where the total rows is fixed and columns
//!   can grow dynamically
//! * [`FixedColumnTable`] struct, where the total columns is fixed and rows
//!   can grow dynamically
//! * [`Table`] trait, which provides the majority of the methods
//!   available to operate on a table
//! * [`iter::CellIter`] trait, which enables examining the row & column
//!   positions of iterators over individual cells in a table as well as zip
//!   an iterator with the position of each cell
//!
pub use crate::{
    impls::{DynamicTable, FixedColumnTable, FixedRowTable, FixedTable},
    iter::CellIter,
    list::*,
    Table,
};
