use std::env;
use std::path::Path;
use std::path::PathBuf;

use gel_tokio::Builder;
use gel_tokio::Config;
use indexmap::IndexMap;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::EXPORTS_IDENT;
use crate::Result;

pub(crate) fn create_gel_config<P: AsRef<Path>>(config_path: Option<P>) -> Result<Config> {
	let config = if let Some(config_path) = config_path {
		Builder::new()
			.without_system()
			.with_env()
			.with_fs()
			.with_auto_project(config_path)
			.build()?
	} else {
		Builder::new().build()?
	};

	Ok(config)
}

#[derive(Debug, Clone, Default)]
pub struct FeatureAliases {
	pub serde: Option<String>,
	pub builder: Option<String>,
	pub query: Option<String>,
}

/// The name of a feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureName {
	Serde,
	Builder,
	Query,
}

type DeriveFeatures = IndexMap<Option<String>, Vec<TokenStream>>;

impl FeatureAliases {
	/// Returns a `TokenStream` of the derive features for the given aliases.
	pub(crate) fn get_derive_features(&self, is_input: bool) -> TokenStream {
		let mut features = DeriveFeatures::new();
		let mut tokens = TokenStream::new();

		if cfg!(feature = "serde") {
			let entry = features.entry(self.serde.clone()).or_default();
			entry.push(quote!(#EXPORTS_IDENT::serde::Serialize));
			entry.push(quote!(#EXPORTS_IDENT::serde::Deserialize));
		}

		if cfg!(feature = "builder") && is_input {
			let entry = features.entry(self.builder.clone()).or_default();
			entry.push(quote!(#EXPORTS_IDENT::typed_builder::TypedBuilder));
		}

		if cfg!(feature = "query") {
			let entry = features.entry(self.query.clone()).or_default();
			entry.push(quote!(#EXPORTS_IDENT::gel_derive::Queryable));
		}

		for (key, derive_tokens) in &features {
			if let Some(key) = key {
				tokens.extend(quote! {
					#[cfg_attr(feature = #key, derive(#(#derive_tokens),*))]
				});
			} else {
				tokens.extend(quote! {
					#[derive(Clone, Debug, #(#derive_tokens),*)]
				});
			}
		}

		tokens
	}

	/// Wrap the provided `TokenStream` annotation if the
	/// `serde` feature is enabled.
	pub(crate) fn wrap_annotation(
		&self,
		feature: FeatureName,
		tokens: &TokenStream,
	) -> TokenStream {
		let empty_tokens = TokenStream::new();

		let key = match feature {
			FeatureName::Serde => {
				if !cfg!(feature = "serde") {
					return empty_tokens;
				}

				&self.serde
			}

			FeatureName::Builder => {
				if !cfg!(feature = "builder") {
					return empty_tokens;
				}

				&self.builder
			}

			FeatureName::Query => {
				if !cfg!(feature = "query") {
					return empty_tokens;
				}

				&self.query
			}
		};

		let Some(key) = key else {
			return quote!(#[#tokens]);
		};

		quote!(#[cfg_attr(feature = #key, #tokens)])
	}
}

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
