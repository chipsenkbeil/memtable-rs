# MemTable: Library to provide an inmemory table for use in Rust

[![Build Status][build_img]][build_lnk]
[![Crates.io][crates_img]][crates_lnk]
[![Docs.rs][doc_img]][doc_lnk]

## Getting Started

### Installation

Import `memtable` into your project by adding the following to `Cargo.toml`:

```toml
[dependencies]
memtable = "0.1"
```

Minimum Rust version is currently **1.51.0**.

### Usage

```rust
use memtable::Table;

let mut table = Table::new();
table.push_row(vec![1, 2, 3]);
table.push_row(vec![4, 5, 6]);

for column in table.columns() {
    for cell in column {
        println!("{}", cell);
    }
}

// Prints
// 1
// 4
// 2
// 5
// 3
// 6
```

If you'd like typed tables, you can generate a table for some data structure
using a derive macro from the `macros` features:

```rust
use memtable::Table;

#[derive(Table)]
struct User {
    name: String,
    age: u8,
}

// Produces a table with typed columns based on above fields
let mut table = UserTable::new();

// Instances of the above struct can be pushed to the table as individual rows
table.push(User { name: "Fred Flintstone".to_string(), age: 51 });

// You can also push a tuple of data
table.push_row(("Wilma Flintstone".to_string(), 47));

// Retrieving data comes in the form of an iterator over a data wrapper
// called UserTableData
let row: Vec<UserTableData> = table.row(0).collect();
assert_eq!(row[0], "Fred Flintstone");
assert_eq!(row[1], 51);
```

See [crate documentation][doc_link] for more examples.

### Features

- **csv** - Enables ability to convert to/from a CSV using a `Table`.
- **serde-1** - Enables ability to serialize `Table`, `Position`, and `CellX`
  data structures.

## License

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
