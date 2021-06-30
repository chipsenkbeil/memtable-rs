use memtable_macros::Table;

#[test]
fn should_support_renaming_table() {
    #[derive(Table)]
    #[table(name = "OtherTable")]
    struct MyStruct {
        field: u8,
    }

    let mut table = OtherTable::new();
    table.push_row(MyStruct { field: 123 });
    assert_eq!(table.row(0), Some(&123));
}

#[test]
fn should_support_forwarding_derive_to_table_and_data() {
    // NOTE: We typically need to derive on both the table
    //       and data, which is why we have a singular test
    #[derive(Table)]
    #[table(derive(Debug), data(derive(Debug)))]
    struct MyStruct {
        field: u8,
    }

    let mut table = MyStructTable::new();
    table.push_row(MyStruct { field: 123 });
    assert_eq!(
        format!("{:?}", table),
        [
            "MyStructTable(DynamicTable {",
            "cells: {Position { row: 0, col: 0 }: Field(123)},",
            "row_cnt: 1,",
            "col_cnt: 1",
            "})",
        ]
        .join(" "),
    );
}

#[test]
fn should_support_skipping_deriving_from_impl_bidirectionally_on_struct() {
    #[derive(Table)]
    #[table(skip_parts)]
    struct MyStruct {
        field1: u8,
        field2: bool,
    }

    // Implementing would conflict with the derived table
    // if we didn't skip it
    impl From<(u8, bool)> for MyStruct {
        fn from((field1, field2): (u8, bool)) -> Self {
            Self { field1, field2 }
        }
    }

    // Implementing would conflict with the derived table
    // if we didn't skip it
    impl From<MyStruct> for (u8, bool) {
        fn from(my_struct: MyStruct) -> Self {
            (my_struct.field1, my_struct.field2)
        }
    }

    let row = MyStruct::from((123, true));
    assert_eq!(row.field1, 123);
    assert!(row.field2);

    let (field1, field2) = row.into();
    assert_eq!(field1, 123);
    assert!(field2);
}

#[test]
fn should_support_renaming_table_data() {
    #[derive(Table)]
    #[table(data(name = "OtherData"))]
    struct MyStruct {
        field1: u8,
        field2: bool,
    }

    let _ = OtherData::Field1(123);
    let _ = OtherData::Field2(true);
}

#[test]
fn should_support_renaming_columns() {
    #[derive(Table)]
    struct MyStruct {
        #[column(name = "number")]
        field1: u8,
        field2: bool,
    }

    let mut table = MyStructTable::new();
    table.push_row(MyStruct {
        field1: 123,
        field2: true,
    });

    // Check get_{} output
    assert_eq!(table.get_cell_number(0), Some(&123));
    assert_eq!(table.get_cell_field2(0), Some(&true));

    // Check get_mut_{} output
    *table.get_mut_cell_number(0).unwrap() = 234;
    *table.get_mut_cell_field2(0).unwrap() = false;

    // Check {}_column output
    assert_eq!(table.column_number().next(), Some(&234));
    assert_eq!(table.column_field2().next(), Some(&false));

    // Check replace_{} output
    assert_eq!(table.replace_cell_number(0, 123), Some(234));
    assert_eq!(table.replace_cell_field2(0, true), Some(false));

    // Check into_{}_column output
    assert!(MyStructTable::new().into_column_number().next().is_none());
    assert!(MyStructTable::new().into_column_field2().next().is_none());
}

#[test]
fn should_support_indexing_columns() {
    #[derive(Table)]
    struct MyStruct {
        #[column(indexed)]
        field1: u8,
        field2: bool,
    }

    // NOTE: This currently does nothing, but when it does we'll want to update
    //       this test with whatever logic is appropriate! Reserved here to
    //       ensure that it compiles to enable future-forward development
    //       where users can go ahead and mark columns as indexed and expect
    //       performance improvements later on
}
