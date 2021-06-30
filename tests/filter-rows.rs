#![cfg(feature = "predicates")]
use memtable::predicates::prelude::*;
use memtable::prelude::*;

#[test]
fn test() {
    let mut table = DynamicTable::new();
    table.push_row(vec![1, 2, 3]);
    table.push_row(vec![4, 5, 6]);
    table.push_row(vec![7, 8, 9]);

    table.filter_rows(predicate::function(|row| row.find(|x| x == 3).is_some()))
}
