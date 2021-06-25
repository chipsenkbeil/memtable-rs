# MemTable: Library to provide an inmemory table for use in Rust

[![Build Status][build_img]][build_lnk]
[![Crates.io][crates_img]][crates_lnk]
[![Docs.rs][doc_img]][doc_lnk]
[![entity: rustc 1.49+]][Rust 1.49]
[![entity_macros: rustc 1.49+]][Rust 1.49]

## Getting Started

### Installation

Import `memtable` into your project by adding the following to `Cargo.toml`:

```toml
[dependencies]
memtable = "0.1"
```

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

[build_img]: https://github.com/chipsenkbeil/memtable/workflows/CI/badge.svg
[build_lnk]: https://github.com/chipsenkbeil/memtable/actions
[crates_img]: https://img.shields.io/crates/v/memtable.svg
[crates_lnk]: https://crates.io/crates/memtable
[doc_img]: https://docs.rs/memtable/badge.svg
[doc_lnk]: https://docs.rs/memtable
