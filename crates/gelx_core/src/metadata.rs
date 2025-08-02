use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use base64::prelude::*;
use gel_tokio::Builder;
use gel_tokio::Config;
use gel_tokio::InstanceName;
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use serde::Deserialize;
use serde::Serialize;
use syn::Ident;
use toml_edit::DocumentMut;
use toml_edit::Item;
use typed_builder::TypedBuilder;

use crate::GelxCoreError;
use crate::GelxCoreResult;
use crate::gelx_error;

/// The metadata for the `gelx` crate. This can either be specified in the
/// `Cargo.toml` file or via CLI arguments.
///
/// Refer to the main `gelx` crate [readme.md](https://github.com/ifiokjr/gelx/blob/main/readme.md#configuration) for all the configuration options.
///
/// ```bash
/// gelx generate
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, TypedBuilder, PartialEq)]
#[builder(field_defaults(
	default,
	setter(into, strip_option(ignore_invalid, fallback_suffix = "_opt"))
))]
pub struct GelxMetadata {
	#[builder(default = default_queries_path())]
	#[serde(default = "default_queries_path")]
	pub queries_path: PathBuf,
	#[builder(default = GelxFeatures::default())]
	#[serde(default = "GelxFeatures::default")]
	pub features: GelxFeatures,
	#[builder(default = default_output_path())]
	#[serde(default = "default_output_path")]
	pub output_path: PathBuf,
	#[builder(default = default_input_struct_name())]
	#[serde(default = "default_input_struct_name")]
	pub input_struct_name: String,
	#[builder(default = default_output_struct_name())]
	#[serde(default = "default_output_struct_name")]
	pub output_struct_name: String,
	#[builder(default = default_query_function_name())]
	#[serde(default = "default_query_function_name")]
	pub query_function_name: String,
	#[builder(default = default_transaction_function_name())]
	#[serde(default = "default_transaction_function_name")]
	pub transaction_function_name: String,
	#[builder(default = default_query_constant_name())]
	#[serde(default = "default_query_constant_name")]
	pub query_constant_name: String,
	#[builder(default = default_exports_alias())]
	#[serde(default = "default_exports_alias")]
	pub exports_alias: String,
	#[builder(default = default_struct_derive_macros())]
	#[serde(default = "default_struct_derive_macros")]
	pub struct_derive_macros: Vec<String>,
	#[builder(default = default_struct_derive_macros())]
	#[serde(default = "default_struct_derive_macros")]
	pub scalar_derive_macros: Vec<String>,
	#[builder(default = default_enum_derive_macros())]
	#[serde(default = "default_enum_derive_macros")]
	pub enum_derive_macros: Vec<String>,
	#[builder(default)]
	#[serde(default)]
	pub gel_config_path: Option<PathBuf>,
	#[builder(default)]
	#[serde(default)]
	pub gel_instance: Option<String>,
	#[builder(default)]
	#[serde(default)]
	pub gel_branch: Option<String>,
	/// The directory of the root of the rust crate. The folder containing the
	/// parent `Cargo.toml` file.
	#[builder(default)]
	#[serde(skip, default)]
	pub root_path: Option<PathBuf>,
	/// Whether this metadata was created from a `build.rs` build script.
	/// Currently this does nothing but in the future this might be used to
	/// further customize the generated code.
	#[builder(default)]
	#[serde(skip, default)]
	pub from_build_script: bool,
}

impl TryFrom<&str> for GelxMetadata {
	type Error = GelxCoreError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let doc = value.parse::<DocumentMut>()?;

		let metadata = if toml_has_path(doc.as_item(), vec!["package", "metadata", "gelx"]) {
			let metadata_str = doc["package"]["metadata"]["gelx"].to_string();
			toml::from_str::<Self>(&metadata_str)?
		} else {
			Self::default()
		};

		Ok(metadata)
	}
}

impl TryFrom<&String> for GelxMetadata {
	type Error = GelxCoreError;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl Display for GelxMetadata {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.try_to_string().map_err(|_| std::fmt::Error)?)
	}
}

impl GelxMetadata {
	/// Attempts to create a new `GelxMetadata` instance from the `Cargo.toml`
	/// file in the given path.
	///
	/// This will also set the `root_path` field to the path of the root of the
	/// rust crate.
	pub fn try_new<P: AsRef<Path>>(path: P) -> GelxCoreResult<Self> {
		let root = get_package_root(path)?;

		let toml_str: String = fs::read_to_string(root.join("Cargo.toml"))?;
		let mut metadata = Self::try_from(toml_str.as_str())?;

		metadata.root_path = Some(root);

		Ok(metadata)
	}

	pub fn try_from_base64(value: impl AsRef<str>) -> GelxCoreResult<Self> {
		let toml_bytes = BASE64_STANDARD.decode(value.as_ref())?;
		let toml_str = String::from_utf8(toml_bytes)?;
		let metadata = Self::try_from(&toml_str)?;

		Ok(metadata)
	}

	/// Create a `Config` instance from the `GelxMetadata`.
	pub fn gel_config(&self) -> GelxCoreResult<Config> {
		let mut builder = Builder::new();

		if let Some(ref instance) = self.gel_instance {
			builder = builder.instance(InstanceName::from_str(instance)?);
		}

		if let Some(ref branch) = self.gel_branch {
			builder = builder.branch(branch);
		}

		let config = if let Some(ref config_path) = self.gel_config_path {
			builder
				.without_system()
				.with_env()
				.with_fs()
				.with_auto_project(config_path)
				.build()?
		} else {
			builder.build()?
		};

		Ok(config)
	}

	pub fn input_struct_ident(&self) -> Ident {
		format_ident!("{}", self.input_struct_name)
	}

	pub fn output_struct_ident(&self) -> Ident {
		format_ident!("{}", self.output_struct_name)
	}

	pub fn query_constant_ident(&self) -> Ident {
		format_ident!("{}", self.query_constant_name)
	}

	pub fn query_function_ident(&self) -> Ident {
		format_ident!("{}", self.query_function_name)
	}

	pub fn transaction_function_ident(&self) -> Ident {
		format_ident!("{}", self.transaction_function_name)
	}

	pub fn exports_alias_ident(&self) -> Ident {
		format_ident!("{}", self.exports_alias)
	}

	pub fn struct_derive_macro_paths(&self) -> Vec<syn::Path> {
		self.struct_derive_macros
			.iter()
			.filter_map(|s| syn::parse_str::<syn::Path>(s).ok())
			.collect()
	}

	pub fn scalar_derive_macro_paths(&self) -> Vec<syn::Path> {
		self.scalar_derive_macros
			.iter()
			.filter_map(|s| syn::parse_str::<syn::Path>(s).ok())
			.collect()
	}

	pub fn enum_derive_macro_paths(&self) -> Vec<syn::Path> {
		self.enum_derive_macros
			.iter()
			.filter_map(|s| syn::parse_str::<syn::Path>(s).ok())
			.collect()
	}

	pub fn try_to_string(&self) -> GelxCoreResult<String> {
		toml::to_string(self).map_err(Into::into)
	}

	pub fn try_to_base64(&self) -> GelxCoreResult<String> {
		let toml_str = self.try_to_string()?;
		let base64_str = BASE64_STANDARD.encode(&toml_str);

		Ok(base64_str)
	}
}

impl Default for GelxMetadata {
	fn default() -> Self {
		Self::builder().build()
	}
}

fn default_queries_path() -> PathBuf {
	PathBuf::from("queries")
}

fn default_output_path() -> PathBuf {
	PathBuf::from("src/db")
}

fn default_input_struct_name() -> String {
	"Input".to_string()
}

fn default_output_struct_name() -> String {
	"Output".to_string()
}

fn default_query_function_name() -> String {
	"query".to_string()
}

fn default_transaction_function_name() -> String {
	"transaction".to_string()
}

fn default_query_constant_name() -> String {
	"QUERY".to_string()
}

fn default_exports_alias() -> String {
	"__g".to_string()
}

fn default_struct_derive_macros() -> Vec<String> {
	vec!["::std::fmt::Debug".into(), "::core::clone::Clone".into()]
}

fn default_enum_derive_macros() -> Vec<String> {
	vec![
		"::std::fmt::Debug".into(),
		"::core::clone::Clone".into(),
		"::core::marker::Copy".into(),
	]
}

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From, PartialEq)]
#[serde(untagged)]
pub enum GelxFeatureOptions {
	/// Enable the feature with the given alias.
	///
	/// ```toml
	/// [package.metadata.gelx]
	/// features = { strum = "ssr" }
	/// ```
	///
	/// The above will enable the `strum` feature only when the feature flag
	/// `ssr` is enabled for the consuming crate.
	Alias(String),
	/// Can be used to disable the feature without an alias.
	///
	/// ```toml
	/// [package.metadata.gelx]
	/// features = { strum = false, builder = false }
	/// ```
	///
	/// The above will disable the `strum` and `builder` features entirely.
	Enabled(bool),
}

impl From<&str> for GelxFeatureOptions {
	fn from(value: &str) -> Self {
		GelxFeatureOptions::Alias(value.to_string())
	}
}

impl Default for GelxFeatureOptions {
	fn default() -> Self {
		GelxFeatureOptions::Enabled(true)
	}
}

impl GelxFeatureOptions {
	pub fn is_enabled(&self) -> bool {
		match self {
			GelxFeatureOptions::Alias(_) => true,
			GelxFeatureOptions::Enabled(val) => *val,
		}
	}

	pub fn alias(&self) -> Option<String> {
		match self {
			GelxFeatureOptions::Alias(alias) => Some(alias.to_owned()),
			GelxFeatureOptions::Enabled(_) => None,
		}
	}
}

/// The name of a feature.
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	strum::AsRefStr,
	strum::Display,
	strum::EnumString,
	strum::EnumIs,
	strum::FromRepr,
	strum::IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum FeatureName {
	Serde,
	Builder,
	Query,
	Strum,
}

impl FeatureName {
	pub fn is_enabled(&self) -> bool {
		#[cfg(feature = "serde")]
		if self == &FeatureName::Serde {
			return true;
		}

		#[cfg(feature = "builder")]
		if self == &FeatureName::Builder {
			return true;
		}

		#[cfg(feature = "query")]
		if self == &FeatureName::Query {
			return true;
		}

		#[cfg(feature = "strum")]
		if self == &FeatureName::Strum {
			return true;
		}

		false
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, TypedBuilder, PartialEq)]
#[builder(field_defaults(default, setter(into)))]
pub struct GelxFeatures {
	#[serde(default)]
	pub query: GelxFeatureOptions,
	#[serde(default)]
	pub strum: GelxFeatureOptions,
	#[serde(default)]
	pub builder: GelxFeatureOptions,
	#[serde(default)]
	pub serde: GelxFeatureOptions,
}

impl GelxFeatures {
	pub(crate) fn get_derive_features(
		&self,
		features: &[FeatureName],
		exports_ident: &Ident,
		derive_macro_paths: &[syn::Path],
		is_input: bool,
		is_macro: bool,
	) -> TokenStream {
		let mut features_map = IndexMap::<Option<String>, Vec<TokenStream>>::new();
		let mut tokens = TokenStream::new();
		let mut extra_tokens = TokenStream::new();
		features_map.insert(None, vec![quote!(#(#derive_macro_paths),*)]);

		for feature in features {
			if !self.is_enabled(*feature, is_macro) {
				continue;
			}

			match feature {
				FeatureName::Serde => {
					let entry = features_map.entry(self.serde.alias()).or_default();
					entry.push(quote!(#exports_ident::serde::Serialize));
					entry.push(quote!(#exports_ident::serde::Deserialize));
				}
				FeatureName::Builder => {
					if is_input {
						let entry = features_map.entry(self.builder.alias()).or_default();
						entry.push(quote!(#exports_ident::typed_builder::TypedBuilder));
						extra_tokens.extend(self.wrap_annotation(
							FeatureName::Builder,
							&quote!(builder(crate_module_path=#exports_ident::typed_builder)),
							is_macro,
						));
					}
				}
				FeatureName::Query => {
					let entry = features_map.entry(self.query.alias()).or_default();
					entry.push(quote!(#exports_ident::gel_derive::Queryable));
					extra_tokens.extend(self.wrap_annotation(
						FeatureName::Query,
						&quote!(gel(crate_path = #exports_ident::gel_protocol)),
						is_macro,
					));
				}
				FeatureName::Strum => {
					let entry = features_map.entry(self.strum.alias()).or_default();
					entry.push(quote!(#exports_ident::strum::AsRefStr));
					entry.push(quote!(#exports_ident::strum::Display));
					entry.push(quote!(#exports_ident::strum::EnumString));
					entry.push(quote!(#exports_ident::strum::EnumIs));
					entry.push(quote!(#exports_ident::strum::FromRepr));
					entry.push(quote!(#exports_ident::strum::IntoStaticStr));

					let strum_crate = format!("{exports_ident}::strum");
					extra_tokens.extend(self.wrap_annotation(
						FeatureName::Strum,
						&quote!(strum(crate = #strum_crate)),
						is_macro,
					));
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
					#[derive(#(#derive_tokens),*)]
				});
			}
		}

		tokens.extend(extra_tokens);

		tokens
	}

	/// Returns a `TokenStream` of the derive features for a struct.
	pub(crate) fn get_struct_derive_features(
		&self,
		exports_ident: &Ident,
		derive_macro_paths: &[syn::Path],
		is_input: bool,
		is_macro: bool,
	) -> TokenStream {
		self.get_derive_features(
			&[FeatureName::Serde, FeatureName::Builder, FeatureName::Query],
			exports_ident,
			derive_macro_paths,
			is_input,
			is_macro,
		)
	}

	/// Returns a `TokenStream` of the derive features for an enum.
	pub(crate) fn get_enum_derive_features(
		&self,
		exports_ident: &Ident,
		derive_macro_paths: &[syn::Path],
		is_macro: bool,
	) -> TokenStream {
		self.get_derive_features(
			&[FeatureName::Serde, FeatureName::Query, FeatureName::Strum],
			exports_ident,
			derive_macro_paths,
			false,
			is_macro,
		)
	}

	pub(crate) fn is_enabled(&self, feature: FeatureName, is_macro: bool) -> bool {
		match feature {
			FeatureName::Serde => self.serde.is_enabled() && (!is_macro || feature.is_enabled()),
			FeatureName::Builder => {
				self.builder.is_enabled() && (!is_macro || feature.is_enabled())
			}
			FeatureName::Query => self.query.is_enabled() && (!is_macro || feature.is_enabled()),
			FeatureName::Strum => self.strum.is_enabled() && (!is_macro || feature.is_enabled()),
		}
	}

	pub(crate) fn alias(&self, feature: FeatureName) -> Option<String> {
		match feature {
			FeatureName::Serde => self.serde.alias(),
			FeatureName::Builder => self.builder.alias(),
			FeatureName::Query => self.query.alias(),
			FeatureName::Strum => self.strum.alias(),
		}
	}

	/// Wrap the provided `TokenStream` annotation if the specified feature is
	/// enabled.
	pub(crate) fn wrap_annotation(
		&self,
		feature: FeatureName,
		tokens: &TokenStream,
		is_macro: bool,
	) -> TokenStream {
		let empty_tokens = TokenStream::new();

		if !self.is_enabled(feature, is_macro) {
			return empty_tokens;
		}

		let Some(key) = self.alias(feature) else {
			return quote!(#[#tokens]);
		};

		quote!(#[cfg_attr(feature = #key, #tokens)])
	}

	/// Create an annotation for the given feature if it has an alias defined.
	/// `#[cfg(feature = "ssr")]`
	pub(crate) fn annotate(&self, feature: FeatureName, is_macro: bool) -> TokenStream {
		let empty_tokens = TokenStream::new();

		if !self.is_enabled(feature, is_macro) {
			return empty_tokens;
		}

		let Some(alias) = self.alias(feature) else {
			return empty_tokens;
		};

		quote!(#[cfg(feature = #alias)])
	}
}

pub fn get_package_root<P: AsRef<Path>>(path: P) -> GelxCoreResult<PathBuf> {
	let path_ancestors = path.as_ref().ancestors();

	for p in path_ancestors {
		let has_cargo = fs::read_dir(p)?.any(|p| p.unwrap().file_name() == *"Cargo.toml");

		if has_cargo {
			return Ok(PathBuf::from(p));
		}
	}

	Err(gelx_error!("Root directory for rust project not found."))
}

fn toml_has_path(doc: &Item, keys: Vec<&str>) -> bool {
	let mut item = doc;
	for key in keys {
		if item.get(key).is_none() {
			return false;
		}
		item = &item[key];
	}

	true
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_gel_config() {
		let metadata = GelxMetadata::builder()
			.gel_config_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../gel.toml"))
			.gel_instance("gelx".to_string())
			.gel_branch("main".to_string())
			.build();

		metadata.gel_config().unwrap();
	}

	#[test]
	fn test_try_from_base64() {
		let expected = GelxMetadata::default();
		let base64 = expected.try_to_base64().unwrap();
		let metadata = GelxMetadata::try_from_base64(&base64).unwrap();

		assert_eq!(expected, metadata);
	}
}
