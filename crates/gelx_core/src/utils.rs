use std::env;
use std::path::Path;
use std::path::PathBuf;

use proc_macro2::Span;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::Result;

/// Taken from <https://github.com/launchbadge/sqlx/blob/f69f370f25f099fd5732a5383ceffc76f724c482/sqlx-macros-core/src/common.rs#L1C1-L37C2>
pub fn resolve_path(path: impl AsRef<Path>, error_span: Span) -> syn::Result<PathBuf> {
	let path = path.as_ref();

	if path.is_absolute() {
		return Err(syn::Error::new(
			error_span,
			"absolute paths will only work on the current machine",
		));
	}

	// requires `proc_macro::SourceFile::path()` to be stable
	// https://github.com/rust-lang/rust/issues/54725
	if path.is_relative()
		&& !path
			.parent()
			.map_or(false, |parent| !parent.as_os_str().is_empty())
	{
		return Err(syn::Error::new(
			error_span,
			"paths relative to the current file's directory are not currently supported",
		));
	}

	let base_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| {
		syn::Error::new(
			error_span,
			"CARGO_MANIFEST_DIR is not set; please use Cargo to build",
		)
	})?;
	let base_dir_path = Path::new(&base_dir);

	Ok(base_dir_path.join(path))
}

/// Will format the given source code using `rustfmt`.
pub async fn rustfmt(source: &str) -> Result<String> {
	let source = prettify(source)?;

	let mut process = Command::new("rustfmt")
		.args(["--emit", "stdout"])
		.stdin(std::process::Stdio::piped())
		.stdout(std::process::Stdio::piped())
		.spawn()?;

	let mut stdin = process.stdin.take().unwrap();
	stdin.write_all(source.as_bytes()).await?;
	stdin.flush().await?;
	drop(stdin);

	let result = String::from_utf8(process.wait_with_output().await?.stdout).map_err(|_| {
		std::io::Error::new(std::io::ErrorKind::InvalidData, "Rustfmt output not UTF-8")
	})?;

	Ok(result)
}

/// Will format the given source code using `prettyplease`.
pub fn prettify(source: &str) -> syn::Result<String> {
	Ok(prettyplease::unparse(&syn::parse_str(source)?))
}

#[cfg(test)]
mod tests {
	use assert2::check;

	use super::*;

	#[tokio::test]
	async fn can_format_file() -> Result<()> {
		let content = "struct Foo { content: String, allowed: bool, times: u64 }";
		let formatted = rustfmt(content).await?;

		// formatting changes based on the version of rust used, so can't check for
		// exact output
		check!(formatted != content);

		Ok(())
	}

	#[tokio::test]
	async fn error_when_formatting_invalid_rust() {
		let content = "struct Foo { content: String, allowed: bool, times: u64,,,,, INVALID}";
		let result = rustfmt(content).await;

		check!(result.is_err(), "result should be an error");
	}
}
