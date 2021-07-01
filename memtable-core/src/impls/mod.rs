#[cfg(any(feature = "alloc", feature = "std"))]
mod dynamic;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use dynamic::DynamicTable;

mod fixed;
pub use fixed::FixedTable;

#[cfg(any(feature = "alloc", feature = "std"))]
mod col;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use col::FixedColumnTable;

#[cfg(any(feature = "alloc", feature = "std"))]
mod row;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use row::FixedRowTable;
