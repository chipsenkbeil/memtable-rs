use memtable_core::Table;
use memtable_macros::Table;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Eq, Table)]
struct MyTuple(bool, usize);

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

#[test]
fn should_support_retrieving_column_names() {
    assert_eq!(MyTupleTable::COLUMN_NAMES, &["0", "1"]);
}

#[test]
fn should_support_retrieving_columns_by_name() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    let column: Vec<bool> = table
        .column_by_name("0")
        .unwrap()
        .filter_map(|x| x.as_0().copied())
        .collect();
    assert_eq!(column, vec![false, true]);

    let column: Vec<usize> = table
        .column_by_name("1")
        .unwrap()
        .filter_map(|x| x.as_1().copied())
        .collect();
    assert_eq!(column, vec![123, 999]);

    assert!(table.column_by_name("???").is_none());
}

#[test]
fn should_support_converting_into_columns_by_name() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    let column: Vec<bool> = table
        .into_column_by_name("0")
        .unwrap()
        .filter_map(MyTupleTableData::into_0)
        .collect();
    assert_eq!(column, vec![false, true]);

    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    let column: Vec<usize> = table
        .into_column_by_name("1")
        .unwrap()
        .filter_map(MyTupleTableData::into_1)
        .collect();
    assert_eq!(column, vec![123, 999]);

    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    assert!(table.into_column_by_name("???").is_none());
}

#[test]
fn should_support_retrieving_typed_rows() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    let mut rows = table.rows();
    assert_eq!(rows.next(), Some((&false, &123)));
    assert_eq!(rows.next(), Some((&true, &999)));
    assert!(rows.next().is_none());
}

#[test]
fn should_support_retrieving_a_single_typed_row() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    assert_eq!(table.row(0), Some((&false, &123)));
    assert_eq!(table.row(1), Some((&true, &999)));
    assert!(table.row(2).is_none());
}

#[test]
fn should_support_inserting_typed_rows() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    // Insert at beginning
    table.insert_row(0, (true, 456));
    {
        let mut rows = table.rows();
        assert_eq!(rows.next(), Some((&true, &456)));
        assert_eq!(rows.next(), Some((&false, &123)));
        assert_eq!(rows.next(), Some((&true, &999)));
        assert!(rows.next().is_none());
    }

    // Insert at end
    table.insert_row(table.row_cnt(), (false, 0));
    {
        let mut rows = table.rows();
        assert_eq!(rows.next(), Some((&true, &456)));
        assert_eq!(rows.next(), Some((&false, &123)));
        assert_eq!(rows.next(), Some((&true, &999)));
        assert_eq!(rows.next(), Some((&false, &0)));
        assert!(rows.next().is_none());
    }

    // Insert in middle
    table.insert_row(table.row_cnt() / 2, (true, 789));
    {
        let mut rows = table.rows();
        assert_eq!(rows.next(), Some((&true, &456)));
        assert_eq!(rows.next(), Some((&false, &123)));
        assert_eq!(rows.next(), Some((&true, &789)));
        assert_eq!(rows.next(), Some((&true, &999)));
        assert_eq!(rows.next(), Some((&false, &0)));
        assert!(rows.next().is_none());
    }
}

#[test]
fn should_support_pushing_typed_rows_to_end() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    let mut rows = table.rows();
    assert_eq!(rows.next(), Some((&false, &123)));
    assert_eq!(rows.next(), Some((&true, &999)));
    assert!(rows.next().is_none());
}

#[test]
fn should_support_removing_typed_rows() {
    let mut table = MyTupleTable::new();

    // Remove when empty
    assert!(table.remove_row(0).is_none());
    assert!(table.rows().next().is_none());

    // Populate data
    table.push_row((false, 1));
    table.push_row((false, 2));
    table.push_row((false, 3));
    table.push_row((false, 4));
    table.push_row((false, 5));

    // Remove from the beginning
    assert_eq!(table.remove_row(0), Some(MyTuple { 0: false, 1: 1 }));
    {
        let mut rows = table.rows();
        assert_eq!(rows.next(), Some((&false, &2)));
        assert_eq!(rows.next(), Some((&false, &3)));
        assert_eq!(rows.next(), Some((&false, &4)));
        assert_eq!(rows.next(), Some((&false, &5)));
        assert!(rows.next().is_none());
    }

    // Remove from the end
    assert_eq!(
        table.remove_row(table.row_cnt() - 1),
        Some(MyTuple { 0: false, 1: 5 })
    );
    {
        let mut rows = table.rows();
        assert_eq!(rows.next(), Some((&false, &2)));
        assert_eq!(rows.next(), Some((&false, &3)));
        assert_eq!(rows.next(), Some((&false, &4)));
        assert!(rows.next().is_none());
    }

    // Remove from the middle
    assert_eq!(
        table.remove_row(table.row_cnt() / 2),
        Some(MyTuple { 0: false, 1: 3 })
    );
    {
        let mut rows = table.rows();
        assert_eq!(rows.next(), Some((&false, &2)));
        assert_eq!(rows.next(), Some((&false, &4)));
        assert!(rows.next().is_none());
    }
}

#[test]
fn should_support_removing_typed_rows_from_end() {
    let mut table = MyTupleTable::new();

    // Remove when empty
    assert!(table.pop_row().is_none());
    assert!(table.rows().next().is_none());

    // Populate data
    table.push_row((false, 1));
    table.push_row((false, 2));
    table.push_row((false, 3));
    table.push_row((false, 4));
    table.push_row((false, 5));

    // Remove from the end
    assert_eq!(table.pop_row(), Some(MyTuple { 0: false, 1: 5 }));
    {
        let mut rows = table.rows();
        assert_eq!(rows.next(), Some((&false, &1)));
        assert_eq!(rows.next(), Some((&false, &2)));
        assert_eq!(rows.next(), Some((&false, &3)));
        assert_eq!(rows.next(), Some((&false, &4)));
        assert!(rows.next().is_none());
    }
}

#[test]
fn should_support_retrieving_typed_columns() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    let mut column = table.column_0();
    assert_eq!(column.next(), Some(&false));
    assert_eq!(column.next(), Some(&true));
    assert!(column.next().is_none());

    let mut column = table.column_1();
    assert_eq!(column.next(), Some(&123));
    assert_eq!(column.next(), Some(&999));
    assert!(column.next().is_none());
}

#[test]
fn should_support_converting_into_typed_columns() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    let mut column = table.into_column_0();
    assert_eq!(column.next(), Some(false));
    assert_eq!(column.next(), Some(true));
    assert!(column.next().is_none());

    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    let mut column = table.into_column_1();
    assert_eq!(column.next(), Some(123));
    assert_eq!(column.next(), Some(999));
    assert!(column.next().is_none());
}

#[test]
fn should_support_replacing_individual_cells() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    assert_eq!(table.replace_cell_0(0, true), Some(false));
    {
        let mut column = table.column_0();
        assert_eq!(column.next(), Some(&true));
        assert_eq!(column.next(), Some(&true));
        assert!(column.next().is_none());
    }

    assert_eq!(table.replace_cell_1(1, 0usize), Some(999));
    {
        let mut column = table.column_1();
        assert_eq!(column.next(), Some(&123));
        assert_eq!(column.next(), Some(&0));
        assert!(column.next().is_none());
    }

    assert_eq!(table.replace_cell_1(2, 999usize), None);
    {
        assert_eq!(table.row_cnt(), 2);
        assert_eq!(table.col_cnt(), 2);

        let mut column = table.column_1();
        assert_eq!(column.next(), Some(&123));
        assert_eq!(column.next(), Some(&0));
        assert!(column.next().is_none());
    }
}

#[test]
fn should_support_retrieving_individual_cells() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    assert_eq!(table.get_cell_0(0), Some(&false));
    assert_eq!(table.get_cell_0(1), Some(&true));
    assert_eq!(table.get_cell_0(2), None);

    assert_eq!(table.get_cell_1(0), Some(&123));
    assert_eq!(table.get_cell_1(1), Some(&999));
    assert_eq!(table.get_cell_1(2), None);
}

#[test]
fn should_support_mutating_individual_cells() {
    let mut table = MyTupleTable::new();
    table.push_row((false, 123));
    table.push_row((true, 999));

    *table.get_mut_cell_0(0).unwrap() = true;
    *table.get_mut_cell_0(1).unwrap() = false;
    assert!(table.get_mut_cell_0(2).is_none());

    *table.get_mut_cell_1(0).unwrap() = 999;
    *table.get_mut_cell_1(1).unwrap() = 123;
    assert!(table.get_mut_cell_1(2).is_none());

    assert_eq!(table.get_cell_0(0), Some(&true));
    assert_eq!(table.get_cell_0(1), Some(&false));
    assert_eq!(table.get_cell_0(2), None);

    assert_eq!(table.get_cell_1(0), Some(&999));
    assert_eq!(table.get_cell_1(1), Some(&123));
    assert_eq!(table.get_cell_1(2), None);
}

#[test]
fn should_support_trying_to_convert_from_untyped_table() {
    // If data is in right columns and is not missing anything,
    // conversion should work fine
    {
        let mut table = memtable::DynamicTable::new();
        table.push_row(vec![MyTupleTableData::_0(false), MyTupleTableData::_1(123)]);
        table.push_row(vec![MyTupleTableData::_0(true), MyTupleTableData::_1(999)]);

        let table = MyTupleTable::try_from(table).unwrap();
        let mut rows = table.rows();
        assert_eq!(rows.next(), Some((&false, &123)));
        assert_eq!(rows.next(), Some((&true, &999)));
        assert!(rows.next().is_none());
    }

    // If data is not in right order in terms of column types, should fail
    {
        let mut table = memtable::DynamicTable::new();
        table.push_row(vec![MyTupleTableData::_1(123), MyTupleTableData::_0(false)]);
        table.push_row(vec![MyTupleTableData::_1(999), MyTupleTableData::_0(true)]);
        assert!(MyTupleTable::try_from(table).is_err());
    }

    // If data is missing in places, should fail
    {
        let mut table = memtable::DynamicTable::new();
        table.push_row(vec![MyTupleTableData::_0(false), MyTupleTableData::_1(123)]);
        table.push_row(vec![MyTupleTableData::_0(true)]);
        assert!(MyTupleTable::try_from(table).is_err());
    }
}
