use memtable_core::Table;
use memtable_macros::Table;

#[test]
fn should_support_tuple_structs() {
    #[derive(Table)]
    struct MyTuple(u8, String);

    let mut table = MyTupleTable::new();
    table.push_row(MyTuple(123, String::from("hello world")));
}

#[test]
fn should_support_tuple_structs_with_generics() {
    #[derive(Table)]
    struct MyTuple<A, B>(A, B);

    let mut table = MyTupleTable::new();
    table.push_row(MyTuple(123, "hello world"));
}

#[test]
fn should_support_tuple_structs_with_lifetimes() {
    #[derive(Table)]
    struct MyTuple<'a>(&'a u8, &'a str);

    let mut table = MyTupleTable::new();

    let data = (123, String::from("hello world"));
    table.push_row(MyTuple(&data.0, &data.1));
}
