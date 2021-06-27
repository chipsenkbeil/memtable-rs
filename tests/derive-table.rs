#![cfg(feature = "macros")]
use memtable::Table;

#[derive(Table)]
struct User {
    name: String,
    age: u8,
}

#[test]
fn test() {
    // Derives a new struct, User{Table}, that can contain instances of User
    // that are broken up into their individual fields
    let mut table = UserTable::new();

    // Inserting is straightforward as a User is considered a singular row
    table.push_row(User {
        name: "Fred Flintstone".to_string(),
        age: 51,
    });

    // You can also pass in a tuple of the fields in order of declaration
    table.push_row(("Wilma Flintstone".to_string(), 47));

    // Retrieval by row will provide the fields by ref as a tuple
    let (name, age) = table.row(0).unwrap();
    assert_eq!(name, "Fred Flintstone");
    assert_eq!(*age, 51);

    // Tables of course provide a variety of other methods to inspect data
    assert_eq!(
        table.name_column().collect::<Vec<&String>>(),
        vec!["Fred Flintstone", "Wilma Flintstone"],
    );
}
