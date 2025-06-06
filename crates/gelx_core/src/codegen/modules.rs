use std::hash::Hash;
use std::path::Path;
use std::path::PathBuf;

use check_keyword::CheckKeyword;
use heck::ToPascalCase;
use heck::ToSnakeCase;
use indexmap::IndexMap;
use indexmap::indexmap;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use syn::Ident;
use tokio::fs;
use uuid::Uuid;

use super::*;
use crate::GelxCoreError;
use crate::GelxCoreResult;
use crate::GelxMetadata;
use crate::Type;
use crate::prettify;

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
			let globals_tokens = generate_globals(self.metadata, self.globals_ref);
			tokens.extend(globals_tokens);
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
