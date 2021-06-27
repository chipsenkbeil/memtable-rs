//! # memtable
//!
//! [![Build Status](https://travis-ci.org/Peternator7/strum.svg?branch=master)](https://travis-ci.org/Peternator7/strum)
//! [![Latest Version](https://img.shields.io/crates/v/strum.svg)](https://crates.io/crates/strum)
//! [![Rust Documentation](https://docs.rs/strum/badge.svg)](https://docs.rs/strum)
//!
//! Strum is a set of macros and traits for working with
//! enums and strings easier in Rust.
//!
//! The full version of the README can be found on
//! [Github](https://github.com/chipsenkbeil/memtable-rs).
//!
//! # Including memtable in Your Project
//!
//! Import memtable following lines to your Cargo.toml.
//!
//! ```toml
//! [dependencies]
//! memtable = "0.1"
//! ```
//!
//! ```
//!

pub use memtable_core::*;

#[cfg(feature = "macros")]
pub use memtable_macros::*;
