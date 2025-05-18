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
use typed_builder::TypedBuilder;

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

#[derive(Debug, Clone, Default, TypedBuilder)]
#[builder(field_defaults(default, setter(into, strip_option(fallback_suffix = "_opt"))))]
pub struct FeatureAliases {
	pub serde: Option<String>,
	pub builder: Option<String>,
	pub query: Option<String>,
	pub strum: Option<String>,
}

/// The name of a feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureName {
	Serde,
	Builder,
	Query,
	Strum,
}

type DeriveFeatures = IndexMap<Option<String>, Vec<TokenStream>>;

impl FeatureAliases {
	fn get_derive_features(
		&self,
		features: &[FeatureName],
		is_input: bool,
		is_copy: bool,
	) -> TokenStream {
		let mut features_map = DeriveFeatures::new();
		let mut tokens = TokenStream::new();

		if is_copy {
			features_map.insert(None, vec![quote!(Copy)]);
		}

		for feature in features {
			match feature {
				FeatureName::Serde => {
					let entry = features_map.entry(self.serde.clone()).or_default();
					entry.push(quote!(#EXPORTS_IDENT::serde::Serialize));
					entry.push(quote!(#EXPORTS_IDENT::serde::Deserialize));
				}
				FeatureName::Builder => {
					if is_input {
						let entry = features_map.entry(self.builder.clone()).or_default();
						entry.push(quote!(#EXPORTS_IDENT::typed_builder::TypedBuilder));
					}
				}
				FeatureName::Query => {
					let entry = features_map.entry(self.query.clone()).or_default();
					entry.push(quote!(#EXPORTS_IDENT::gel_derive::Queryable));
				}
				FeatureName::Strum => {
					let entry = features_map.entry(self.strum.clone()).or_default();
					entry.push(quote!(#EXPORTS_IDENT::strum::AsRefStr));
					entry.push(quote!(#EXPORTS_IDENT::strum::Display));
					entry.push(quote!(#EXPORTS_IDENT::strum::EnumString));
					entry.push(quote!(#EXPORTS_IDENT::strum::EnumIs));
					entry.push(quote!(#EXPORTS_IDENT::strum::FromRepr));
					entry.push(quote!(#EXPORTS_IDENT::strum::IntoStaticStr));
				}
			}
		}

		for (key, derive_tokens) in &features_map {
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

	/// Returns a `TokenStream` of the derive features for a struct.
	pub(crate) fn get_struct_derive_features(&self, is_input: bool) -> TokenStream {
		self.get_derive_features(
			&[FeatureName::Serde, FeatureName::Builder, FeatureName::Query],
			is_input,
			false,
		)
	}

	/// Returns a `TokenStream` of the derive features for an enum.
	pub(crate) fn get_enum_derive_features(&self) -> TokenStream {
		self.get_derive_features(&[FeatureName::Query, FeatureName::Strum], false, true)
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

			FeatureName::Strum => {
				if !cfg!(feature = "strum") {
					return empty_tokens;
				}

				&self.strum
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
