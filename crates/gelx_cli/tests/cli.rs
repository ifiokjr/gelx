use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

const CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn cli() -> Command {
	Command::new(insta_cmd::get_cargo_bin("gelx"))
}

fn example_path() -> PathBuf {
	PathBuf::from(CRATE_DIR).join("../../examples/gelx_example")
}

#[test]
fn invalid_cwd() {
	insta_cmd::assert_cmd_snapshot!(
		cli()
			.arg("generate")
			.arg("--cwd")
			.arg("../../examples/gelx_example/does/not/exist")
			.stderr(Stdio::null())
	);
}

#[test]
fn cwd() {
	insta_cmd::assert_cmd_snapshot!(
		cli()
			.arg("generate")
			.arg("--cwd")
			.arg("../../examples/gelx_example")
			.stderr(Stdio::null())
	);
}

#[test]
fn generate_stdout() {
	insta_cmd::assert_cmd_snapshot!(
		cli()
			.arg("generate")
			.arg("--json")
			.current_dir(example_path())
			.stderr(Stdio::null())
	);
}

#[test]
fn generate_files() {
	insta_cmd::assert_cmd_snapshot!(
		cli()
			.arg("generate")
			.current_dir(example_path())
			.stderr(Stdio::null())
	);
}

#[test]
fn check() {
	insta_cmd::assert_cmd_snapshot!(
		cli()
			.arg("check")
			.current_dir(example_path())
			.stderr(Stdio::null())
	);
}
