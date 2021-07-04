# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.2.0] - 2021-07-03

### Added

- `no_std` support via the `alloc` and `std` features (involved refactoring
  `Table` trait to use new `list::DynamicList` and `list::FixedList`)
- New `sled` feature that provides a new table wrapper `SledTable` that uses
  [sled](https://github.com/spacejam/sled) as a replication and persistence
  layer for an inmemory table
- `Capacity` indicator to `Table` trait via new `max_row_capacity`
  and `max_column_capacity` methods
- New macro attribute `mode` at the table level that supports using a different
  table to power the typed version generated. Three types are `dynamic`
  (default), `fixed_column` where the max column is set to number of fields
  in struct, and `fixed` where the max column is set the same as `fixed_column`
  but you also specify a maximum number of rows:

  ```rust
  use memtable::Table;

  // Creates a table that can have a dynamic number of rows and columns
  #[derive(Table)]
  #[table(mode = "dynamic")]
  struct DataSource1 {
    field1: String,
    field2: bool,
  }

  // Creates a table that can have a dynamic number of rows with a fixed
  // set of columns: in this case COL is set to 2
  #[derive(Table)]
  #[table(mode = "fixed_column")]
  struct DataSource1 {
    field1: String,
    field2: bool,
  }

  // Creates a table that has a fixed number of rows and columns, where the
  // max columns is determined by the number of fields (2) and the rows is
  // explicitly highlighted (such as 25 rows)
  #[derive(Table)]
  #[table(mode(fixed(rows = "25")))]
  struct DataSource1 {
    field1: String,
    field2: bool,
  }
  ```

### Changed

- Renamed `MemTable` to `DynamicTable`
- Updated `serde::Deserialize` of `FixedTable`, `FixedRowTable`, and
  `FixedColumnTable` to allocate inline instead of creating an initial array
  that is pre-allocated
    - Internally added `utils::try_make_array` and `utils::try_make_table_array`
      that allocate an array by creating one element at a time, supporting
      failures to create new elements and automatically handling proper
      dropping of a partially-created array
- Readjusted `FixedTable`, `FixedRowTable`, and `FixedColumnTable` to work in
  a more expected manor where they maintain virtual capacities that grow &
  shrink, do not return "uninitialized data" when accessing out of bounds
  (return `None` instead of `Some(Default::default()))`, and properly truncate
  values by filling in with a default when requested
- Renamed `Table` trait's `get_cell` and `get_mut_cell` to `cell` and `mut_cell`
- Renamed `Table` derived methods
    1. Remove `get_cell_` and `get_mut_cell_` from in front of column accessors
    2. Change `replace_cell_{...}` to just `replace_{...}`
    3. Flip `column_{...}` and `into_column_{...}` to `{...}_column` and
       `into_{...}_column`
- Renamed `Table` trait's `set_row_capacity` and `set_column_capacity` to
  `set_preferred_row_cnt` and `set_preferred_col_cnt`

### Fixed

- `Table` derive macro no longer panics when provided tuple structs

## [0.1.0] - 2021-06-27

### Added

- `memtable` crate containing:
    - **Documentation**: main crate documentation
    - **Re-exporting**: `memtable-core` and `memtable-macros` (under `macros` feature)
- `memtable-core` crate containing:
    - **Tables**: `MemTable`, `FixedTable`, `FixedRowTable`, and `FixedColumnTable`
    - **Traits**: `Table` and `CellIter`
    - **Extensions**: `FromCsv`/`ToCsv` and `Cell2` through `Cell26`
- `memtable-macros` crate containing:
    - **Derive**: `Table`
