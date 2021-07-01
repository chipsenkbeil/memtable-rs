# memtable - Inmemory tables for use in Rust

[![Build Status][build_img]][build_lnk]
[![Crates.io][crates_img]][crates_lnk]
[![Docs.rs][doc_img]][doc_lnk]
[![memtable Minimum Supported Rust Version][memtable_msrv_img]][memtable_msrv_lnk]
[![memtable-macros Minimum Supported Rust Version][memtable_macros_msrv_img]][memtable_macros_msrv_lnk]

## Overview

memtable provides a collection of table-oriented features for use inmemory.

This crate acts as the aggregator of all subcrates such as `memtable-core`
and `memtable-macros` and should be the only crate imported when using
features from either.

## Installation

At its core, you can import the dependency by adding the following to your
`Cargo.toml`:

```toml
[dependencies]
memtable = "0.2"
```

In the situation where you would like to derive typed tables based on
user-defined structs, you can include the `macros` feature:

```toml
[dependencies]
memtable = { version = "0.2", features = ["macros"] }
```

### no-std support

Additionally, this library has support for `no_std`, both with and without
inclusion of `alloc`. This is done by turning off default features (`std` is
the only default feature). From there, if you would like to include `alloc`
support, then add that feature:

```toml
[dependencies]
# For no_std without alloc support
memtable = { version = "0.2", default-features = false }

# For no_std with alloc support
memtable = { version = "0.2", default-features = false, features = ["alloc"] }
```

Please keep in mind that relying only on the `core` made available by default
will limit your table options to `FixedTable`. You are also still able to use
the `macros` feature to derive typed tables, but you must explicitly set the
mode to `fixed`.

## Usage

Most often, you will want to import the `prelude` to bring in relevant
traits and structs:

```rust
use memtable::prelude::*;

// Create a 2x3 (row x column) table of integers
let mut table = FixedTable::from([
    [1, 2, 3],
    [4, 5, 6],
]);

// Examine one of the values, replace it, and examine again
assert_eq!(table[(1, 2)], 6);
table[(1, 2)] = 999;
assert_eq!(table[(1, 2)], 999);
```

## The Tables

In the core library, you will find four primary tables:

- `DynamicTable`: table with a dynamic capacity for rows & columns
- `FixedTable`: table with a fixed capacity for rows & columns
- `FixedRowTable`: table with a fixed capacity for rows & dynamic capacity for columns
- `FixedColumnTable`: table with a dynamic capacity for rows & fixed capacity for columns

## The Traits

- `Table`: primary trait that exposes majority of common operations
  to perform on tables
- `CellIter`: common trait that table iterators focused on
  individual cells that enables zipping with a cell's
  position and getting the current row & column of
  the iterator

## The Features

Alongside the essentials, the library also provides several features that
provide extensions to the table arsenal:

- **alloc**: opts into the alloc crate in the situation that `no_std` is in effect
- **csv**: enables `FromCsv` (convert CSV into an inmemory table) and `ToCsv`
  (convert an inmemory table to CSV)
- **cell**: enables `Cell2` and more up to `Cell26`, which represent generic
  enums that can be used as the data type for a table to enable multiple
  data types within a table (e.g. `DynamicTable<Cell2<String, bool>>`)
- **macros**: enables `Table` macro to derive new struct that implements the
  `Table` trait to be able to store some struct into a dedicated, inmemory table
- **serde**: enables *serde* support on all table & cell implementations
- **sled**: enables `SledTable`, which provides persistent storage on top of
  other tables via the sled database
- **std**: *(enabled by default)* opts into the std library; if removed then
  `no_std` is enabled

## The Macros

Currently, there is a singular macro, `Table`, which is used to
derive a table to contain zero or more of a specific struct.

```rust
use memtable::Table;

#[derive(Table)]
struct User {
    name: String,
    age: u8,
}

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
    table.column_name().collect::<Vec<&String>>(),
    vec!["Fred Flintstone", "Wilma Flintstone"],
);
```

## The License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in vimvar by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>

[build_img]: https://github.com/chipsenkbeil/memtable-rs/workflows/CI/badge.svg
[build_lnk]: https://github.com/chipsenkbeil/memtable-rs/actions
[crates_img]: https://img.shields.io/crates/v/memtable.svg
[crates_lnk]: https://crates.io/crates/memtable
[doc_img]: https://docs.rs/memtable/badge.svg
[doc_lnk]: https://docs.rs/memtable
[memtable_msrv_img]: https://img.shields.io/badge/memtable-rustc_1.51+-blueviolet.svg
[memtable_macros_msrv_img]: https://img.shields.io/badge/memtable_macros-rustc_1.51+-blueviolet.svg
[memtable_msrv_lnk]: https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html
[memtable_macros_msrv_lnk]: https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html
