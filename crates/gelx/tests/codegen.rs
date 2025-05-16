use std::path::PathBuf;
use std::thread;

use gelx_core::*;
use proc_macro2::Span;
use rstest::fixture;
use rstest::rstest;

macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
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

#[rstest]
#[case::str("select 'i ❤️ gel'")]
#[case::bool("select true")]
#[case::int64("select 3")]
#[case::float64("select 314e-2")]
#[case::bigint("select 42n")]
#[case::decimal("select 42e+100n")]
#[case::uuid("select <uuid>'a5ea6360-75bd-4c20-b69c-8f317b0d2857'")]
#[case::datetime("select <datetime>'1999-03-31T15:17:00Z'")]
#[case::duration("select <duration>'45.6 seconds'")]
#[case::array("select [1, 2, 3]")]
#[case::tuple("select ('Apple', 7, true)")]
#[case::named_tuple("select (fruit := 'Apple', quantity := 3.14, fresh := true)")]
#[case::set(r#"select {"set", "of", "strings"}"#)]
#[case::empty_set("select <int64>{}")]
#[case::input_shape(
	"select { my_string := RelationshipType.Follow, my_number := 42, several_numbers := {1, 2, \
	 3}, array := [1, 2, 3] };"
)]
#[case::input_shape_with_args(r#"select { hello := "world", custom := <str>$custom }"#)]
#[case::object_shape("select Team {**}")]
#[case::object_shape_with_args(
	"select Team {**} filter .name like <str>$starts_with ++ '%' and .description like '%' ++ \
	 <str>$ends_with;"
)]
#[case::types_query(TYPES_QUERY)]
// #[case::range("select range(0, 10)")] // TODO: `Range` doesn't implement `Queryable` yet.
// #[case::bytes("select b'bina\\x01ry'")] // TODO: bytes don't implement `DecodeScalar` yet.
#[tokio::test]
#[rustversion::attr(not(nightly), ignore = "requires nightly")]
async fn codegen_literals(testname: String, #[case] query: &str) -> Result<()> {
	set_snapshot_suffix!("{}", testname);

	let relative_path = format!("tests/compile/codegen/{testname}.rs");
	let descriptor = get_descriptor(query, Option::<&str>::None).await?;
	let code = generate_rust_from_query(&descriptor, "example", query, &FeatureAliases::default())?;
	let content = prettify(&code.to_string())?;

	// Check that the snapshot hasn't changed.
	insta::assert_snapshot!(&content);

	// Ensure that the produced rust is valid.
	prepare_compile_test(&content, &relative_path).await?;

	Ok(())
}

#[rstest]
#[case("insert_user")]
#[case("remove_user")]
#[tokio::test]
async fn codegen_files(#[case] path: &str) -> Result<()> {
	set_snapshot_suffix!("{}", path);

	let query_path = resolve_path(format!("queries/{path}.edgeql"), Span::call_site())?;
	let relative_path = format!("tests/compile/codegen/{path}.rs");
	let query = tokio::fs::read_to_string(&query_path).await?;
	let descriptor = get_descriptor(&query, Option::<&str>::None).await?;
	let code =
		generate_rust_from_query(&descriptor, "example", &query, &FeatureAliases::default())?;
	let content = prettify(&code.to_string())?;

	// Check that the snapshot hasn't changed.
	insta::assert_snapshot!(&content);

	// Ensure that the produced rust is valid.
	prepare_compile_test(&content, &relative_path).await?;

	Ok(())
}

const CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");

async fn prepare_compile_test(content: &str, relative_path: &str) -> Result<()> {
	let is_ci = std::env::var("CI")
		.ok()
		.is_some_and(|v| ["1", "true"].contains(&v.as_str()));

	let path = PathBuf::from(CRATE_DIR).join(relative_path);
	let generated = generate_contents(content)?;

	let should_update = match tokio::fs::read_to_string(&path).await {
		Ok(current) => current != generated,
		Err(_) => true,
	};

	if should_update {
		assert2::assert!(!is_ci, "attempted updating compilation tests in CI");
		tokio::fs::write(&path, generated).await?;
	}

	Ok(())
}

fn generate_contents(content: &str) -> Result<String> {
	let updated = format!("fn main() {{\n{content}\n}}");
	Ok(prettify(&updated)?)
}
