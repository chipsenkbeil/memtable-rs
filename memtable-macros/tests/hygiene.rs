#![no_implicit_prelude]
// NOTE: This file exists to validate that the prelude can be excluded and the
//       macros produce code with proper pathing; no tests are needed here as
//       this is purely validating that the macros are hygienic via compilation

// NOTE: Macros is looking for memtable, so we map our core crate since that's
//       actually what is used underneath
extern crate memtable_core as memtable;

// Struct should be supported with all primitive types
#[derive(::memtable_macros::Table)]
struct MyRow {
    field1: ::std::primitive::bool,
    field2: ::std::primitive::char,
    field3: ::std::primitive::f32,
    field4: ::std::primitive::f64,
    field5: ::std::primitive::i128,
    field6: ::std::primitive::i16,
    field7: ::std::primitive::i32,
    field8: ::std::primitive::i64,
    field9: ::std::primitive::i8,
    field10: ::std::primitive::isize,
    field11: &'static ::std::primitive::str,
    field12: ::std::primitive::u128,
    field13: ::std::primitive::u16,
    field14: ::std::primitive::u32,
    field15: ::std::primitive::u64,
    field16: ::std::primitive::u8,
    field17: ::std::primitive::usize,
    field18: ::std::string::String,
}

// Struct should support generics
#[derive(::memtable_macros::Table)]
struct GenericRow<A, B> {
    field1: A,
    field2: B,
}

// Struct sohuld support lifetimes
#[derive(::memtable_macros::Table)]
struct LifetimeRow<'a> {
    field1: &'a ::std::primitive::str,
    field2: &'a ::std::path::Path,
}

// These traits exist to make sure we properly import using
// ::std::primitive::<TYPE> instead of purely <TYPE>
//
// Only works for Rust 1.43.0+
#[allow(non_camel_case_types)]
trait bool {}
#[allow(non_camel_case_types)]
trait char {}
#[allow(non_camel_case_types)]
trait f32 {}
#[allow(non_camel_case_types)]
trait f64 {}
#[allow(non_camel_case_types)]
trait i128 {}
#[allow(non_camel_case_types)]
trait i16 {}
#[allow(non_camel_case_types)]
trait i32 {}
#[allow(non_camel_case_types)]
trait i64 {}
#[allow(non_camel_case_types)]
trait i8 {}
#[allow(non_camel_case_types)]
trait isize {}
#[allow(non_camel_case_types)]
trait str {}
#[allow(non_camel_case_types)]
trait u128 {}
#[allow(non_camel_case_types)]
trait u16 {}
#[allow(non_camel_case_types)]
trait u32 {}
#[allow(non_camel_case_types)]
trait u64 {}
#[allow(non_camel_case_types)]
trait u8 {}
#[allow(non_camel_case_types)]
trait usize {}
