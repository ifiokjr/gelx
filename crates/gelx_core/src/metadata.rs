use std::fs;
use std::path::Path;
use std::path::PathBuf;

use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use serde::Serialize;
use toml_edit::DocumentMut;
use toml_edit::Item;
use typed_builder::TypedBuilder;

use crate::EXPORTS_IDENT;
use crate::GelxCoreError;
use crate::GelxCoreResult;
use crate::gelx_error;

/// The metadata for the `gelx` crate. This can either be specified in the
/// `Cargo.toml` file or via CLI arguments.
///
/// ```toml
/// [package.metadata.gelx]
/// ## The location of the queries relative to the root of the crate.
/// queries = "./queries"
///
/// ## The features to enable and their aliases. By default all features are enabled.
/// ## To disable a feature set it to false. The available features are:
/// ## - query
/// ## - serde
/// ## - strum
/// ## - builder
/// features = { query = "ssr", strum = "ssr", builder = "ssr" }
///
/// ## The location of the generated code when using the `gelx` cli.
/// output_file = "./src/gelx_generated.rs"
///
/// ## The name of the arguments input struct. Will be transformed to PascalCase.
/// input_struct_name = "Input"
///
/// ## The name of the exported output struct for generated queries. Will be transformed to PascalCase.
/// output_struct_name = "Output"
///
/// ## The name of the query function exported.
/// query_function_name = "query"
///
/// ## The name of the transaction function exported.
/// transaction_function_name = "transaction"
///
/// ## The relative path to the `gel` config file. This is optional and if not provided the `gel`
/// ## config will be read from the environment variables.
/// # gel_config_path = "./gel.toml"
///
/// ## The name of the `gel` instance to use. This is optional and if not provided the environment
/// ## variable `$GEL_INSTANCE` will be used.
/// # gel_instance = "$GEL_INSTANCE"
///
/// ## The name of the `gel` branch to use. This is optional and if not provided the environment
/// ## variable `$GEL_BRANCH` will be used.
/// # gel_branch = "$GEL_BRANCH"
/// ```
///
/// ```bash
/// gelx generate
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(
	default,
	setter(into, strip_option(ignore_invalid, fallback_suffix = "_opt"))
))]
pub struct GelxMetadata {
	#[builder(default = default_queries_path())]
	#[serde(default = "default_queries_path")]
	pub queries: PathBuf,
	#[builder(default = GelxFeatures::default())]
	#[serde(default = "GelxFeatures::default")]
	pub features: GelxFeatures,
	#[builder(default = default_output_path())]
	#[serde(default = "default_output_path")]
	pub output: PathBuf,
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
	#[builder(default)]
	#[serde(default)]
	pub gel_config_path: Option<PathBuf>,
	#[builder(default)]
	#[serde(default)]
	pub gel_instance: Option<String>,
	#[builder(default)]
	#[serde(default)]
	pub gel_branch: Option<String>,
}

impl GelxMetadata {
	pub fn try_new<P: AsRef<Path>>(path: P) -> GelxCoreResult<Self> {
		let root = get_package_root(path)?;
		let toml_str: String = fs::read_to_string(root.join("Cargo.toml"))?;
		let doc = toml_str.parse::<DocumentMut>()?;

		let mut metadata = if toml_has_path(doc.as_item(), vec!["package", "metadata", "gelx"]) {
			let metadata_str = doc["package"]["metadata"]["gelx"].to_string();
			toml::from_str::<Self>(&metadata_str)?
		} else {
			Self::default()
		};

		metadata.queries = root.join(metadata.queries);
		metadata.output = root.join(metadata.output);
		metadata.gel_config_path = metadata.gel_config_path.map(|p| root.join(p));

		Ok(metadata)
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
	PathBuf::from("src/gelx_generated.rs")
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

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Default, TypedBuilder)]
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
	fn get_derive_features(
		&self,
		features: &[FeatureName],
		is_input: bool,
		is_copy: bool,
		is_macro: bool,
	) -> TokenStream {
		let mut features_map = IndexMap::<Option<String>, Vec<TokenStream>>::new();
		let mut tokens = TokenStream::new();

		if is_copy {
			features_map.insert(None, vec![quote!(Copy)]);
		}

		for feature in features {
			if !self.is_enabled(*feature, is_macro) {
				continue;
			}

			match feature {
				FeatureName::Serde => {
					let entry = features_map.entry(self.serde.alias()).or_default();
					entry.push(quote!(#EXPORTS_IDENT::serde::Serialize));
					entry.push(quote!(#EXPORTS_IDENT::serde::Deserialize));
				}
				FeatureName::Builder => {
					if is_input {
						let entry = features_map.entry(self.builder.alias()).or_default();
						entry.push(quote!(#EXPORTS_IDENT::typed_builder::TypedBuilder));
					}
				}
				FeatureName::Query => {
					let entry = features_map.entry(self.query.alias()).or_default();
					entry.push(quote!(#EXPORTS_IDENT::gel_derive::Queryable));
				}
				FeatureName::Strum => {
					let entry = features_map.entry(self.strum.alias()).or_default();
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
	pub(crate) fn get_struct_derive_features(&self, is_input: bool, is_macro: bool) -> TokenStream {
		self.get_derive_features(
			&[FeatureName::Serde, FeatureName::Builder, FeatureName::Query],
			is_input,
			false,
			is_macro,
		)
	}

	/// Returns a `TokenStream` of the derive features for an enum.
	pub(crate) fn get_enum_derive_features(&self, is_macro: bool) -> TokenStream {
		self.get_derive_features(
			&[FeatureName::Serde, FeatureName::Query, FeatureName::Strum],
			false,
			true,
			is_macro,
		)
	}

	pub(crate) fn is_enabled(&self, feature: FeatureName, is_macro: bool) -> bool {
		match feature {
			FeatureName::Serde => self.serde.is_enabled() && (!is_macro || cfg!(feature = "serde")),
			FeatureName::Builder => {
				self.builder.is_enabled() && (!is_macro || cfg!(feature = "builder"))
			}
			FeatureName::Query => self.query.is_enabled() && (!is_macro || cfg!(feature = "query")),
			FeatureName::Strum => self.strum.is_enabled() && (!is_macro || cfg!(feature = "strum")),
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

	/// Wrap the provided `TokenStream` annotation if the
	/// `serde` feature is enabled.
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

	/// Wrap the provided `TokenStream` annotation.
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
