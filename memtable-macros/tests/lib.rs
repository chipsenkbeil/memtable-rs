// NOTE: Macros is looking for memtable, so we map our core crate since that's
//       actually what is used underneath
extern crate memtable_core as memtable;

// #[derive(::memtable_macros::Table)]
// struct MyTuple(u8, String);

mod derive;
mod hygiene;

/// Runs all ui tests - note that all tests run through trybuild must be done
/// in one test method unless we manually run cargo test with a single thread
///
/// UI tests only run on nightly
///
/// https://github.com/dtolnay/trybuild/issues/58
/// https://github.com/dtolnay/trybuild/issues/6
#[rustversion::attr(not(nightly), ignore)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/**/*.rs");
}
