//! # The MemDynamicTable Prelude
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
//! * [`MemDynamicTable`] struct, which is the core table available from
//!   this crate that acts as a table that can grow and shrink dynamically
//! * [`MemFixedTable`] struct - available with Rust 1.51+ - provides a fixed-sized
//!   counterpart to [`MemDynamicTable`] where the table is pre-allocated internally
//!   using a 2D array
//! * [`MemFixedRowTable`] struct, where the total rows is fixed and columns
//!   can grow dynamically
//! * [`MemFixedColumnTable`] struct, where the total columns is fixed and rows
//!   can grow dynamically
//! * [`Table`] trait, which provides the majority of the methods
//!   available to operate on a table
//! * [`iter::CellIter`] trait, which enables examining the row & column
//!   positions of iterators over individual cells in a table as well as zip
//!   an iterator with the position of each cell
//! * [`RefOrOwned`] struct, which acts as a bridge between retrieving data
//!   from a table with the intention of getting a reference (some may only
//!   support new data such as `sled`)
//! * [`MutRefOrOwned`] struct, which acts as a bridge between retrieving data
//!   from a table with the intention of getting a mutable reference (some may
//!   only support new data such as `sled`)
//!
pub use crate::{
    impls::{MemDynamicTable, MemFixedColumnTable, MemFixedRowTable, MemFixedTable},
    iter::CellIter,
    MutRefOrOwned, RefOrOwned, Table,
};
