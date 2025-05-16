#![cfg(all(feature = "query", feature = "serde"))]
use std::thread;

use rstest::fixture;
use trybuild::TestCases;

#[fixture]
fn t() -> TestCases {
	TestCases::new()
}

#[fixture]
pub fn testname() -> String {
	thread::current()
		.name()
		.unwrap()
		.split("::")
		.last()
		.unwrap()
		.to_string()
}

#[cfg_attr(miri, ignore = "incompatible with miri")]
#[test]
fn check_compilation() {
	let t = TestCases::new();

	t.compile_fail("tests/compile/macros/*.rs");
	t.pass("tests/compile/codegen/*.rs");
}
