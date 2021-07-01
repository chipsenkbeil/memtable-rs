#![cfg(feature = "macros")]
#![cfg_attr(not(feature = "std"), no_std)]
use memtable::Table;

// NOTE: Using fixed table underneath to support pure core test
#[derive(Table)]
#[table(mode(fixed(rows = "5")))]
struct User {
    name: &'static str,
    age: u8,
}

#[test]
fn derive_table() {
    // Derives a new struct, User{Table}, that can contain instances of User
    // that are broken up into their individual fields
    let mut table = UserTable::new();

    // Inserting is straightforward as a User is considered a singular row
    table.push_row(User {
        name: "Fred Flintstone",
        age: 51,
    });

    // You can also pass in a tuple of the fields in order of declaration
    table.push_row(("Wilma Flintstone", 47));

    // Retrieval by row will provide the fields by ref as a tuple
    let (name, age) = table.row(0).unwrap();
    assert_eq!(*name, "Fred Flintstone");
    assert_eq!(*age, 51);

    // Tables of course provide a variety of other methods to inspect data
    let mut names_iter = table.column_name();
    assert_eq!(names_iter.next(), Some(&"Fred Flintstone"));
    assert_eq!(names_iter.next(), Some(&"Wilma Flintstone"));
}
