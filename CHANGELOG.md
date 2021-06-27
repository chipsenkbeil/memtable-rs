# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

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