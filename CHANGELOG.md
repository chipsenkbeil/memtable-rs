# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- New `sled` feature that enables managing tables in [sled](https://github.com/spacejam/sled)
  instead of a mixture of `HashMap<...>` and arrays

### Changed

- Renamed tables
  - `MemTable` to `MemDynamicTable`
  - `FixedTable` to `MemFixedTable`
  - `FixedRowTable` to `MemFixedRowTable`
  - `FixedColumnTable` to `MemFixedColumnTable`
- Updated `serde::Deserialize` of `MemFixedTable`, `MemFixedRowTable`, and
  `MemFixedColumnTable` to allocate inline instead of creating an initial array
  that is pre-allocated
    - Internally added `utils::try_make_array` and `utils::try_make_table_array`
      that allocate an array by creating one element at a time, supporting
      failures to create new elements and automatically handling proper
      dropping of a partially-created array

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
