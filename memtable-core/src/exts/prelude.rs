//! # The DynamicTable Extensions Prelude
//!
//! The memtable library comes with a variety of tools to help with building,
//! parsing, and transforming tables. While these could be brought in via a
//! mixture of `use memtable::*;` and other imports, this prelude serves as
//! the one-stop shop to import required traits, common structs, and more
//! without polluting the namespace with public modules exposed by this crate.
//!
//! # Prelude contents
//!
//! If the `csv` feature is enabled, the prelude re-exports the following:
//!
//! * [`csv::ToCsv`] trait, which enables converting a
//!   table to a CSV
//! * [`csv::FromCsv`] trait, which enables converting
//!   CSV to a table
//!
//! If the `cell` feature is enabled, the prelude re-exports the following:
//!
//! * [`cell::Cell2`] enum, which provides a simple way to
//!   configure a table to have 1 of 2 possible data types
//! * [`cell::Cell3`] enum, which provides a simple way to
//!   configure a table to have 1 of 3 possible data types
//! * [`cell::Cell4`] enum, which provides a simple way to
//!   configure a table to have 1 of 4 possible data types
//! * [`cell::Cell5`] enum, which provides a simple way to
//!   configure a table to have 1 of 5 possible data types
//! * [`cell::Cell6`] enum, which provides a simple way to
//!   configure a table to have 1 of 6 possible data types
//! * [`cell::Cell7`] enum, which provides a simple way to
//!   configure a table to have 1 of 7 possible data types
//! * [`cell::Cell8`] enum, which provides a simple way to
//!   configure a table to have 1 of 8 possible data types
//! * [`cell::Cell9`] enum, which provides a simple way to
//!   configure a table to have 1 of 9 possible data types
//! * [`cell::Cell10`] enum, which provides a simple way to
//!   configure a table to have 1 of 10 possible data types
//! * [`cell::Cell11`] enum, which provides a simple way to
//!   configure a table to have 1 of 11 possible data types
//! * [`cell::Cell12`] enum, which provides a simple way to
//!   configure a table to have 1 of 12 possible data types
//! * [`cell::Cell13`] enum, which provides a simple way to
//!   configure a table to have 1 of 13 possible data types
//! * [`cell::Cell14`] enum, which provides a simple way to
//!   configure a table to have 1 of 14 possible data types
//! * [`cell::Cell15`] enum, which provides a simple way to
//!   configure a table to have 1 of 15 possible data types
//! * [`cell::Cell16`] enum, which provides a simple way to
//!   configure a table to have 1 of 16 possible data types
//! * [`cell::Cell17`] enum, which provides a simple way to
//!   configure a table to have 1 of 17 possible data types
//! * [`cell::Cell18`] enum, which provides a simple way to
//!   configure a table to have 1 of 18 possible data types
//! * [`cell::Cell19`] enum, which provides a simple way to
//!   configure a table to have 1 of 19 possible data types
//! * [`cell::Cell20`] enum, which provides a simple way to
//!   configure a table to have 1 of 20 possible data types
//! * [`cell::Cell21`] enum, which provides a simple way to
//!   configure a table to have 1 of 21 possible data types
//! * [`cell::Cell22`] enum, which provides a simple way to
//!   configure a table to have 1 of 22 possible data types
//! * [`cell::Cell23`] enum, which provides a simple way to
//!   configure a table to have 1 of 23 possible data types
//! * [`cell::Cell24`] enum, which provides a simple way to
//!   configure a table to have 1 of 24 possible data types
//! * [`cell::Cell25`] enum, which provides a simple way to
//!   configure a table to have 1 of 25 possible data types
//! * [`cell::Cell26`] enum, which provides a simple way to
//!   configure a table to have 1 of 26 possible data types
//!
#[cfg(feature = "cell")]
#[cfg_attr(feature = "docs", doc(cfg(cell)))]
#[doc(inline)]
pub use crate::exts::cell::*;

#[cfg(feature = "csv")]
#[cfg_attr(feature = "docs", doc(cfg(csv)))]
#[doc(inline)]
pub use crate::exts::csv::{FromCsv, ToCsv};
