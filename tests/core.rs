#![cfg(not(any(feature = "alloc", feature = "std")))]
#![no_std]
use memtable::prelude::*;

#[test]
fn fixed_table() {
    let mut table = FixedTable::from([[1, 2, 3], [4, 5, 6], [0, 0, 0]]);
    table.set_row_capacity(2);
    table.push_row([7, 8, 9].iter().copied());
    assert_eq!(table, [[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
}
