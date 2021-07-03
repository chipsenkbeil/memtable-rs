/// Contains `CellX` data structures that enable easy multi-type tables
/// by acting as an abstraction of the data sources
#[cfg(feature = "cell")]
#[cfg_attr(feature = "docs", doc(cfg(cell)))]
pub mod cell;

/// Contains traits that enable converting between table and csv data
#[cfg(all(feature = "csv", feature = "std"))]
#[cfg_attr(feature = "docs", doc(cfg(all(csv, std))))]
pub mod csv;

/// Support for using sled as a backing data storage for tables
#[cfg(all(feature = "sled-1", feature = "std"))]
#[cfg_attr(feature = "docs", doc(cfg(all(sled, std))))]
pub mod sled;

/// Contains relevant traits, structs, and more for extensions to tables
pub mod prelude;
