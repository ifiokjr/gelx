use std::hash::Hash;
use std::path::Path;
use std::path::PathBuf;

use check_keyword::CheckKeyword;
use gel_tokio::Client;
use heck::ToPascalCase;
use heck::ToSnakeCase;
use indexmap::IndexMap;
use indexmap::indexmap;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use syn::Ident;
use syn::LitByte;
use tokio::fs;
use uuid::Uuid;

pub use self::globals::*;
pub use self::types::*;
use crate::FeatureName;
use crate::GelxCoreError;
use crate::GelxCoreResult;
use crate::GelxMetadata;
use crate::maybe_uuid_to_token_name;
use crate::prettify;

mod globals;
mod types;

const SYSTEM_NAMESPACES: &[&str] = &["std", "sys", "cfg", "schema", "multirange", "ext"];
fn is_system_namespace(name: impl AsRef<str>) -> bool {
	let name = name.as_ref();
	SYSTEM_NAMESPACES.contains(&name)
}

pub trait ToModuleName {
	fn to_module_name(&self) -> ModuleName;
}

impl<T: AsRef<str>> ToModuleName for T {
	fn to_module_name(&self) -> ModuleName {
		self.as_ref().into()
	}
}

/// The name of a resource in the `gel` database.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleName {
	pub modules: Vec<String>,
	pub name: String,
	pub child: Option<Box<ModuleName>>,
}

impl<T: AsRef<str>> From<T> for ModuleName {
	fn from(s: T) -> Self {
		let mut parent = None;
		let mut value = s.as_ref();

		if value.ends_with('>') {
			let Some((parent_name, child_name)) = value.split_once('<') else {
				panic!("cannot parse module name: `{value}`");
			};

			value = parent_name;
			parent = Some(Box::new(Self::from(
				child_name.trim_end_matches('>').to_string(),
			)));
		}

		let all = value.split("::").collect::<Vec<_>>();
		let name = all.last().unwrap();
		let modules = all
			.iter()
			.take(all.len() - 1)
			.map(ToString::to_string)
			.collect();

		Self {
			modules,
			name: (*name).to_string(),
			child: parent,
		}
	}
}

impl ModuleName {
	pub fn original_name(&self) -> String {
		let modules = self.modules.join("::");

		let parent_name = if modules.is_empty() {
			self.name.clone()
		} else {
			format!("{modules}::{}", self.name)
		};

		if let Some(child) = &self.child {
			format!("{parent_name}<{}>", child.original_name())
		} else {
			parent_name
		}
	}

	pub fn is_system_namespace(&self) -> bool {
		self.modules.first().is_some_and(is_system_namespace)
	}

	pub fn is_user_defined(&self) -> bool {
		!self.is_system_namespace()
	}

	pub fn modules_path(&self) -> GelxCoreResult<syn::Path> {
		let path: syn::Path = syn::parse_str(
			&self
				.modules
				.iter()
				.map(|m| m.to_snake_case().into_safe())
				.collect::<Vec<_>>()
				.join("::"),
		)?;
		Ok(path)
	}

	pub fn name_ident(&self, snake_case: bool) -> Ident {
		if snake_case {
			format_ident!("{}", self.name.to_snake_case().into_safe())
		} else {
			format_ident!("{}", self.name.to_pascal_case().into_safe())
		}
	}
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleOutput {
	/// The path to the module.
	pub path: PathBuf,
	/// The tokens to be written to the module.
	#[serde_as(as = "DisplayFromStr")]
	pub tokens: TokenStream,
}

#[derive(Debug, derive_more::IntoIterator, derive_more::Deref, derive_more::DerefMut)]
pub struct ModuleOutputs(Vec<ModuleOutput>);

impl ModuleOutputs {
	pub fn new(outputs: Vec<ModuleOutput>) -> Self {
		Self(outputs)
	}

	/// Try to read the module outputs from the path.
	pub async fn try_new(path: &PathBuf, base: &PathBuf) -> GelxCoreResult<ModuleOutputs> {
		let mut outputs = Self::new(Vec::new());
		let mut read_dir = fs::read_dir(path).await?;

		while let Some(entry) = read_dir.next_entry().await? {
			let path = entry.path();

			if path.is_dir() {
				outputs.extend(Box::pin(ModuleOutputs::try_new(&path, base)).await?);
				continue;
			}

			let tokens = fs::read_to_string(&path).await?.parse::<TokenStream>()?;

			if let Ok(path) = path.strip_prefix(base) {
				outputs.push(ModuleOutput {
					path: path.to_path_buf(),
					tokens,
				});
			}
		}

		Ok(outputs)
	}

	pub async fn write_to_fs(&self, path: impl AsRef<Path>) -> GelxCoreResult<()> {
		let path = path.as_ref();

		let futures = self.iter().map(|output| {
			async move {
				eprintln!("Writing to {}", output.path.display());
				let content = prettify(&output.tokens.to_string())?;
				let current_path = path.join(&output.path);

				if let Some(parent) = current_path.parent() {
					fs::create_dir_all(parent).await?;
				}

				fs::write(&current_path, content.as_bytes()).await?;

				Ok::<_, GelxCoreError>(())
			}
		});

		futures::future::try_join_all(futures).await?;

		Ok(())
	}

	/// Convert the module outputs to a map of path to code.
	///
	/// The path is the path to the module.
	/// The code is the code of the module.
	pub fn to_map(&self) -> GelxCoreResult<IndexMap<PathBuf, String>> {
		let mut map = IndexMap::new();

		for module_output in self.iter() {
			let path = module_output.path.clone();
			let code = prettify(&module_output.tokens.to_string())?;

			map.insert(path, code);
		}

		Ok(map)
	}

	pub fn append_to_root(&mut self, tokens: &TokenStream) {
		let Some(root) = self.first_mut() else {
			return;
		};

		root.tokens.extend(tokens.clone());
	}

	pub fn push_module(&mut self, path: PathBuf, tokens: TokenStream) {
		self.push(ModuleOutput { path, tokens });
	}
}

#[derive(Debug)]
pub struct ModuleTree<'a> {
	pub root: ModuleNode<'a>,
	pub types: &'a IndexMap<Uuid, Type>,
	pub globals: &'a Vec<GlobalsOutput>,
	pub metadata: &'a GelxMetadata,
}

impl<'a> ModuleTree<'a> {
	pub fn new(
		types_ref: &'a IndexMap<Uuid, Type>,
		globals_ref: &'a Vec<GlobalsOutput>,
		metadata: &'a GelxMetadata,
	) -> Self {
		let mut root = ModuleNode {
			name: String::new(),
			children: indexmap! {},
			types: indexmap! {},
			types_ref,
			globals_ref,
			metadata,
		};

		for (_, type_info) in types_ref {
			let name = type_info.name().to_module_name();

			// TODO: manage the child types which have subtypes later on, for now we just
			// skip them
			if name.child.is_some() {
				continue;
			}

			root.insert(name.modules.as_slice(), type_info);
		}

		Self {
			root,
			types: types_ref,
			globals: globals_ref,
			metadata,
		}
	}

	/// Generate the module outputs.
	///
	/// The module outputs can be used to generate the file modules.
	pub fn generate_modules(&self) -> ModuleOutputs {
		let mut outputs = Vec::new();
		self.root
			.generate_module_output(PathBuf::new(), &mut outputs);

		ModuleOutputs::new(outputs)
	}
}

#[derive(Debug)]
pub struct ModuleNode<'a> {
	pub name: String,
	pub children: IndexMap<String, ModuleNode<'a>>,
	pub types: IndexMap<Uuid, &'a Type>,
	types_ref: &'a IndexMap<Uuid, Type>,
	globals_ref: &'a Vec<GlobalsOutput>,
	metadata: &'a GelxMetadata,
}

impl<'a> ModuleNode<'a> {
	/// Check if the node is the root node.
	///
	/// The root node is the node that has no parent.
	pub fn is_root(&self) -> bool {
		self.name.is_empty()
	}

	/// Insert a type into the module tree.
	///
	/// This is a recursive function that inserts a type into the module tree.
	/// It takes a list of modules and a type information.
	/// The modules are the path to the type in the module tree.
	/// The type information is the type to insert.
	/// The function will insert the type into the module tree.
	pub fn insert(&mut self, modules: &[String], type_info: &'a Type) {
		if modules.is_empty() {
			if !self.types.contains_key(&type_info.id()) {
				self.types.insert(type_info.id(), type_info);
			}

			return;
		}

		let [current, remaining @ ..] = modules else {
			return;
		};

		if let Some(node) = self.children.get_mut(current) {
			node.insert(remaining, type_info);
		} else {
			let mut node = ModuleNode {
				name: current.clone(),
				children: indexmap! {},
				types: indexmap! {},
				types_ref: self.types_ref,
				globals_ref: self.globals_ref,
				metadata: self.metadata,
			};

			node.insert(remaining, type_info);
			self.children.insert(current.clone(), node);
		}
	}

	pub fn generate_module_output(&self, path: PathBuf, outputs: &mut Vec<ModuleOutput>) {
		if !self.is_user_defined() {
			return;
		}

		let current_path = path.join(self.filename());

		outputs.push(ModuleOutput {
			path: current_path.clone(),
			tokens: self.to_token_stream(),
		});

		let child_path = current_path.parent().map_or(path, Path::to_path_buf);

		for child in self.children.values() {
			child.generate_module_output(child_path.clone(), outputs);
		}
	}

	pub fn filename(&self) -> String {
		if self.children.is_empty() {
			self.name.to_snake_case() + ".rs"
		} else if self.is_root() {
			"mod.rs".to_string()
		} else {
			self.name.to_snake_case() + "/mod.rs"
		}
	}

	pub fn safe_name(&self) -> String {
		self.name.to_snake_case().into_safe()
	}

	/// Generate the import tokens for the module.
	fn imports_token_stream(&self) -> TokenStream {
		let mut tokens = TokenStream::new();
		let exports_ident = self.metadata.exports_alias_ident();

		tokens.extend(quote!(
			//! This file is generated by `gelx generate`.
			//! It is not intended for manual editing.
			//! To update it, run `gelx generate`.
			#![cfg_attr(rustfmt, rustfmt_skip)]
			#![allow(unused)]
			#![allow(unused_qualifications)]
			#![allow(clippy::all)]
		));

		if self.is_root() {
			let default_import = self.children.get("default").map(|default_child| {
				let safe_name = format_ident!("{}", default_child.safe_name());
				quote! (pub use #safe_name::*;)
			});

			tokens.extend(quote! {
				use ::gelx::exports as #exports_ident;
				#default_import
			});
		} else {
			tokens.extend(quote!(
				use super::*;
			));
		}

		for (_, node) in &self.children {
			if !node.is_user_defined() {
				continue;
			}

			let filename = node.filename();
			let safe_name = format_ident!("{}", node.safe_name());

			tokens.extend(quote!(
				#[path = #filename]
				pub mod #safe_name;
			));
		}

		tokens
	}

	pub fn is_user_defined(&self) -> bool {
		self.children.values().any(ModuleNode::is_user_defined)
			|| !self.user_defined_types().is_empty()
	}

	pub fn user_defined_types(&self) -> IndexMap<&Uuid, &Type> {
		self.types
			.iter()
			.filter_map(|(id, type_info)| {
				type_info
					.name()
					.to_module_name()
					.is_user_defined()
					.then_some((id, *type_info))
			})
			.collect()
	}

	pub fn to_token_stream(&self) -> TokenStream {
		let mut tokens = TokenStream::new();
		let user_defined_types = self.user_defined_types();

		tokens.extend(self.imports_token_stream());

		if self.is_root() {
			let exports_ident = self.metadata.exports_alias_ident();
			let (fields, setters): (Vec<_>, Vec<_>) = self
				.globals_ref
				.iter()
				.filter_map(|global| {
					let name = &global.name;
					let module_name = name.to_module_name();
					let field_name = module_name.name_ident(true);
					let target = global.target.as_ref()?;
					let type_name = maybe_uuid_to_token_name(&target.id, &exports_ident)?;

					if target.is_from_alias.unwrap_or_default() {
						return None;
					}

					let wrapped_type = if let Some(SchemaCardinality::Many) = global.cardinality {
						quote!(Vec<#type_name>)
					} else {
						quote!(Option<#type_name>)
					};

					let field = quote! {
						pub #field_name: #wrapped_type,
					};

					let setter = quote! {
						modifier.set(#name, self.#field_name);
					};

					Some((field, setter))
				})
				.collect::<Vec<_>>()
				.into_iter()
				.unzip();

			if !fields.is_empty() {
				let derive_macro_paths = self.metadata.struct_derive_macro_paths();
				let struct_derive_tokens = self.metadata.features.get_struct_derive_features(
					&exports_ident,
					&derive_macro_paths,
					true,
					false,
				);
				let typed_builder_annotation = self.metadata.features.wrap_annotation(
					FeatureName::Builder,
					&quote!(builder(field_defaults(
						default,
						setter(into, strip_option(fallback_suffix = "_opt"))
					))),
					false,
				);
				let queryable_annotation =
					self.metadata.features.annotate(FeatureName::Query, false);
				tokens.extend(quote! {
					#struct_derive_tokens
					#typed_builder_annotation
					pub struct Globals {
						#(#fields)*
					}
					#queryable_annotation
					impl #exports_ident::gel_tokio::GlobalsDelta for Globals {
						fn apply(self, modifier: &mut #exports_ident::gel_tokio::state::GlobalsModifier<'_>) {
							#(#setters)*
						}
					}
					#queryable_annotation
					impl Globals {
						/// Create a gel client with the globals.
						pub async fn into_client(self) -> ::core::result::Result<#exports_ident::gel_tokio::Client, #exports_ident::gel_tokio::Error> {
							let client = #exports_ident::gel_tokio::create_client().await?.with_globals(self);
							Ok(client)
						}

						/// Create a gel client with the globals.
						pub async fn to_client(&self) -> ::core::result::Result<#exports_ident::gel_tokio::Client, #exports_ident::gel_tokio::Error> {
							let client = self.clone().into_client().await?;
							Ok(client)
						}
					}
				});
			}
		}

		if user_defined_types.is_empty() {
			return tokens;
		}

		for (_id, type_info) in user_defined_types {
			let module_name = type_info.name().to_module_name();
			match type_info {
				Type::Scalar(scalar_type) => {
					let scalar_tokens = generate_scalar(
						self.metadata,
						scalar_type,
						&module_name,
						self.types_ref,
						false,
					);
					tokens.extend(scalar_tokens);
				}

				Type::Object(object_type) => {
					if object_type.is_abstract {
						// skip for now, we will handle this later
						continue;
					}

					let struct_mod_name = module_name.name_ident(true);
					let struct_tokens = quote! {
						mod #struct_mod_name {
							use super::*;

						}
					};
					tokens.extend(struct_tokens);
				}

				Type::Enum(enum_type) => {
					let enum_tokens = generate_enum(
						self.metadata,
						&enum_type.enum_values,
						&module_name.name,
						false,
					);
					tokens.extend(enum_tokens);
				}

				_ => {}
			}
		}

		tokens
	}
}

/// Generate the custom types.
pub async fn generate_module_outputs(metadata: &GelxMetadata) -> GelxCoreResult<ModuleOutputs> {
	let config = metadata.gel_config()?;
	let client = Client::new(&config);
	let fetched_types = query_types(&client).await?;
	let fetched_globals = query_globals(&client).await?;
	let types = map_fetched_types(&fetched_types);
	let module_tree = ModuleTree::new(&types, &fetched_globals, metadata);
	let outputs = module_tree.generate_modules();

	Ok(outputs)
}

pub(crate) fn generate_enum(
	metadata: &GelxMetadata,
	enum_values: &[String],
	local_name: &str,
	is_macro: bool,
) -> TokenStream {
	let exports_ident = metadata.exports_alias_ident();
	let pascal_local_name = format_ident!("{}", local_name.to_pascal_case().into_safe());
	let enum_values_tokens = enum_values.iter().map(|value| {
		let pascal_value = value.to_pascal_case();
		let annotation = if value == &pascal_value {
			TokenStream::new()
		} else {
			let serde_annotation = metadata.features.wrap_annotation(
				FeatureName::Serde,
				&quote!(serde(rename = #value)),
				is_macro,
			);
			let strum_annotation = metadata.features.wrap_annotation(
				FeatureName::Strum,
				&quote!(strum(serialize = #value)),
				is_macro,
			);

			quote! {
				#serde_annotation
				#strum_annotation
			}
		};
		let enum_value_token = format_ident!("{}", pascal_value);

		quote! {
			#annotation
			#enum_value_token
		}
	});

	let derive_macro_paths = metadata.enum_derive_macro_paths();
	let enum_derive =
		metadata
			.features
			.get_enum_derive_features(&exports_ident, &derive_macro_paths, is_macro);
	let strum_annotation = metadata.features.annotate(FeatureName::Strum, false);
	let enum_tokens = quote! {
		#enum_derive
		pub enum #pascal_local_name {
			#(#enum_values_tokens),*
		}

		#strum_annotation
		impl ::core::convert::From<#pascal_local_name> for #exports_ident::gel_protocol::value::Value {
			fn from(value: #pascal_local_name) -> Self {
				#exports_ident::gel_protocol::value::Value::Enum(value.as_ref().into())
			}
		}
	};

	enum_tokens
}

pub(crate) fn generate_scalar(
	metadata: &GelxMetadata,
	scalar_type: &ScalarType,
	module_name: &ModuleName,
	types: &IndexMap<Uuid, Type>,
	is_macro: bool,
) -> TokenStream {
	let exports_ident = metadata.exports_alias_ident();
	let Some(Type::Scalar(parent_scalar)) = scalar_type.material_id.and_then(|id| types.get(&id))
	else {
		return TokenStream::new();
	};

	let Some(wrapped_struct_type) = maybe_uuid_to_token_name(&parent_scalar.id, &exports_ident)
	else {
		return TokenStream::new();
	};

	let struct_name = module_name.name_ident(false);
	let derive_macro_paths = metadata.struct_derive_macro_paths();
	let struct_derive_tokens = metadata.features.get_derive_features(
		&[FeatureName::Serde, FeatureName::Builder],
		&exports_ident,
		&derive_macro_paths,
		false,
		is_macro,
	);
	let uuid_bytes = scalar_type.id.as_bytes();
	// 1. Convert each byte to a proc_macro2::Literal
	let byte_literals: Vec<LitByte> = uuid_bytes
		.iter()
		.map(|&byte| LitByte::new(byte, Span::call_site()))
		.collect();
	let type_name = &scalar_type.name;
	let queryable_annotation = metadata.features.annotate(FeatureName::Query, false);

	quote! {
		#struct_derive_tokens
		pub struct #struct_name(pub #wrapped_struct_type);

		#queryable_annotation
		impl #exports_ident::gel_protocol::queryable::Queryable for #struct_name {
			type Args = <#wrapped_struct_type as #exports_ident::gel_protocol::queryable::Queryable>::Args;

			fn decode(
				decoder: &#exports_ident::gel_protocol::queryable::Decoder,
				args: &Self::Args,
				buf: &[u8],
			) -> Result<Self, #exports_ident::gel_protocol::errors::DecodeError> {
				Ok(Self(#wrapped_struct_type::decode(decoder, args, buf)?))
			}

			fn check_descriptor(
				ctx: &#exports_ident::gel_protocol::queryable::DescriptorContext,
				type_pos: #exports_ident::gel_protocol::descriptors::TypePos,
			) -> Result<Self::Args, #exports_ident::gel_protocol::queryable::DescriptorMismatch> {
				#exports_ident::check_scalar(
					ctx,
					type_pos,
					#exports_ident::uuid::Uuid::from_bytes([ #( #byte_literals ),* ]),
					#type_name,
				)?;
				Ok(())
			}
		}

		impl ::core::convert::From<#struct_name> for #exports_ident::gel_protocol::value::Value {
			fn from(value: #struct_name) -> Self {
				value.0.into()
			}
		}
		impl ::core::convert::From<#struct_name> for #wrapped_struct_type {
			fn from(value: #struct_name) -> Self {
				value.0
			}
		}
		impl ::core::convert::From<#wrapped_struct_type> for #struct_name {
			fn from(value: #wrapped_struct_type) -> Self {
				#struct_name(value)
			}
		}
		impl ::std::ops::Deref for #struct_name {
			type Target = i32;
			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
		impl ::std::ops::DerefMut for #struct_name {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}
	}
}

mod modname {}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_generate_enum() -> GelxCoreResult<()> {
		let metadata = GelxMetadata::default();

		generate_module_outputs(&metadata).await?;

		Ok(())
	}

	#[test]
	fn test_module_name() {
		let original = "test::test2::Amazing";
		let module_name = ModuleName::from(original);

		assert_eq!(module_name.original_name(), original);
		assert_eq!(
			module_name.modules_path().unwrap(),
			syn::parse_str("test::test2").unwrap()
		);
		assert_eq!(module_name.name_ident(true), format_ident!("amazing"));
		assert_eq!(module_name.name_ident(false), format_ident!("Amazing"));
	}
}
