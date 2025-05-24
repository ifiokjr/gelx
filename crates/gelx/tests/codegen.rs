use std::path::PathBuf;
use std::thread;

use gelx_core::*;
use proc_macro2::Span;
use rstest::fixture;
use rstest::rstest;

#[allow(clippy::perf)]
fn get_features() -> String {
	let mut features = Vec::new();

	#[cfg(feature = "builder")]
	features.push("builder");

	#[cfg(feature = "query")]
	features.push("query");

	#[cfg(feature = "serde")]
	features.push("serde");

	#[cfg(feature = "strum")]
	features.push("strum");

	if !features.is_empty() {
		features.insert(0, "_");
	}

	println!("features: {features:?}");

	features.join("_")
}

macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
				let path = format!($($expr,)*);
        settings.set_snapshot_suffix(format!("{path}{}", get_features()));
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
#[case::enums("select Account { provider }")]
#[case::types_query(TYPES_QUERY)]
// #[case::range("select range(0, 10)")] // TODO: `Range` doesn't implement `Queryable` yet.
// #[case::bytes("select b'bina\\x01ry'")] // TODO: bytes don't implement `DecodeScalar` yet.
#[tokio::test]
#[rustversion::attr(not(nightly), ignore = "requires nightly")]
async fn codegen_literals(testname: String, #[case] query: &str) -> GelxCoreResult<()> {
	set_snapshot_suffix!("{}", testname);

	let metadata = GelxMetadata::default();
	let relative_path = format!("tests/compile/codegen/{testname}{}.rs", get_features());
	let descriptor = get_descriptor(query, &metadata).await?;
	let code = generate_query_token_stream(&descriptor, "example", query, &metadata, true)?;
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
async fn codegen_files(#[case] path: &str) -> GelxCoreResult<()> {
	set_snapshot_suffix!("{}", path);

	let metadata = GelxMetadata::default();
	let query_path = resolve_path(format!("queries/{path}.edgeql"), Span::call_site())?;
	let relative_path = format!("tests/compile/codegen/{path}{}.rs", get_features());
	let query = tokio::fs::read_to_string(&query_path).await?;
	let descriptor = get_descriptor(&query, &metadata).await?;
	let code = generate_query_token_stream(&descriptor, "example", &query, &metadata, true)?;
	let content = prettify(&code.to_string())?;

	// Check that the snapshot hasn't changed.
	insta::assert_snapshot!(&content);

	// Ensure that the produced rust is valid.
	prepare_compile_test(&content, &relative_path).await?;

	Ok(())
}

const CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");

async fn prepare_compile_test(content: &str, relative_path: &str) -> GelxCoreResult<()> {
	#[cfg(all(
		feature = "builder",
		feature = "query",
		feature = "serde",
		feature = "strum"
	))]
	{
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
	}

	Ok(())
}

fn generate_contents(content: &str) -> GelxCoreResult<String> {
	let updated = format!("{content}\n\nfn main() {{}}");
	Ok(prettify(&updated)?)
}

#[tokio::test]
async fn can_generate_enums() -> GelxCoreResult<()> {
	set_snapshot_suffix!("enums");

	let metadata = GelxMetadata::default();
	let outputs = generate_module_outputs(&metadata).await?;
	let json = outputs.to_json_value()?;

	insta::assert_yaml_snapshot!(json);

	Ok(())
}
#[tokio::test]
async fn can_generate_aliased_enums() -> GelxCoreResult<()> {
	set_snapshot_suffix!("aliased_enums");

	let metadata = GelxMetadata::builder()
		.features(
			GelxFeatures::builder()
				.query("ssr")
				.builder("ssr")
				.strum("ssr")
				.build(),
		)
		.build();
	let outputs = generate_module_outputs(&metadata).await?;
	let json = outputs.to_json_value()?;

	insta::assert_yaml_snapshot!(json);

	Ok(())
}
