use std::env;
use std::path::Path;
use std::path::PathBuf;

use proc_macro2::Span;

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
		&& path
			.parent()
			.is_none_or(|parent| parent.as_os_str().is_empty())
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

/// Will format the given source code using `prettyplease`.
pub fn prettify(source: &str) -> syn::Result<String> {
	Ok(prettyplease::unparse(&syn::parse_str(source)?))
}

#[cfg(test)]
mod tests {
	use assert2::check;

	use super::*;
	use crate::GelxCoreResult;

	#[test]
	fn can_format_file() -> GelxCoreResult<()> {
		let content = "struct Foo { content: String, allowed: bool, times: u64 }";
		let formatted = prettify(content)?;

		insta::assert_snapshot!(formatted);

		Ok(())
	}

	#[test]
	fn error_when_formatting_invalid_rust() {
		let content = "struct Foo { content: String, allowed: bool, times: u64,,,,, INVALID}";
		let result = prettify(content);

		check!(result.is_err(), "result should be an error");
	}
}
