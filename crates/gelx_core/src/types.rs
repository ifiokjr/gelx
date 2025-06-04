use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use std::path::PathBuf;

use bitflags::bitflags;
use check_keyword::CheckKeyword;
use gel_derive::Queryable;
use gel_protocol::common::Cardinality;
use gel_tokio::Client;
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

use crate::FeatureName;
use crate::GelxCoreError;
use crate::GelxCoreResult;
use crate::GelxMetadata;
use crate::constants::TYPES_QUERY;
use crate::prettify;

/// Execute the types query to get the types of the current database.
async fn types_query(client: &Client) -> GelxCoreResult<Vec<TypesOutput>> {
	let result = client.query(TYPES_QUERY, &()).await?;

	Ok(result)
}

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
	pub metadata: &'a GelxMetadata,
}

impl<'a> ModuleTree<'a> {
	pub fn new(types_ref: &'a IndexMap<Uuid, Type>, metadata: &'a GelxMetadata) -> Self {
		let mut root = ModuleNode {
			name: String::new(),
			children: indexmap! {},
			types: indexmap! {},
			types_ref,
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

		if user_defined_types.is_empty() {
			return tokens;
		}

		for (_id, type_info) in user_defined_types {
			let module_name = type_info.name().to_module_name();
			match type_info {
				Type::Object(_object_type) => {
					// dbg!(object_type);

					// if object_type.is_abstract {
					// 	// skip for now, we will handle this later
					// 	continue;
					// }

					// let struct_mod_name = module_name.name_ident(true);
					// let struct_tokens = quote! {
					// 	mod #struct_mod_name {
					// 		use super::*;

					// 	}
					// };
					// tokens.extend(struct_tokens);
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
	let fetched_types = types_query(&client).await?;
	let types = map_fetched_types(&fetched_types);
	let module_tree = ModuleTree::new(&types, metadata);
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
		impl From<#pascal_local_name> for #exports_ident::gel_protocol::value::Value {
			fn from(value: #pascal_local_name) -> Self {
				#exports_ident::gel_protocol::value::Value::Enum(value.as_ref().into())
			}
		}
	};

	enum_tokens
}

#[derive(Clone, Debug, Queryable)]
pub struct BasesSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable)]
pub struct UnionOfSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable)]
pub struct IntersectionOfSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable)]
pub struct PointersSetPointersSet {
	pub card: Option<String>,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_computed: Option<bool>,
	pub is_readonly: Option<bool>,
}

impl PointersSetPointersSet {
	pub fn cardinality(&self) -> Cardinality {
		to_cardinality(self.card.as_deref())
	}
}

#[derive(Clone, Debug, Queryable)]
pub struct PointersSet {
	pub card: Option<String>,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: bool,
	pub is_computed: Option<bool>,
	pub is_readonly: Option<bool>,
	pub has_default: bool,
	pub pointers: Vec<PointersSetPointersSet>,
}

impl PointersSet {
	pub fn cardinality(&self) -> Cardinality {
		to_cardinality(self.card.as_deref())
	}
}

#[derive(Clone, Debug, Queryable)]
pub struct ExclusivesSet {
	pub target: Option<String>,
}

#[derive(Clone, Debug, Queryable)]
pub struct BacklinksSet {
	pub card: String,
	pub name: String,
	pub stub: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: Option<bool>,
}

impl BacklinksSet {
	pub fn cardinality(&self) -> Cardinality {
		to_cardinality(Some(&self.card))
	}
}

#[derive(Clone, Debug, Queryable)]
pub struct BacklinkStubsArray {
	pub card: String,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: bool,
}

impl BacklinkStubsArray {
	pub fn cardinality(&self) -> Cardinality {
		to_cardinality(Some(&self.card))
	}
}

#[derive(Clone, Debug, Queryable)]
pub struct TupleElementsSet {
	pub target_id: Uuid,
	pub name: Option<String>,
}

#[derive(Clone, Debug, Queryable)]
pub struct TypesOutput {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: Option<bool>,
	pub kind: String,
	pub enum_values: Option<Vec<String>>,
	pub is_seq: bool,
	pub material_id: Option<Uuid>,
	pub bases: Vec<BasesSet>,
	pub union_of: Vec<UnionOfSet>,
	pub intersection_of: Vec<IntersectionOfSet>,
	pub pointers: Vec<PointersSet>,
	pub exclusives: Vec<ExclusivesSet>,
	pub backlinks: Vec<BacklinksSet>,
	pub backlink_stubs: Vec<BacklinkStubsArray>,
	pub array_element_id: Option<Uuid>,
	pub tuple_elements: Vec<TupleElementsSet>,
	pub multirange_element_id: Option<Uuid>,
}

impl TypesOutput {
	fn pointers_map(&self) -> HashMap<String, Pointer> {
		self.pointers
			.iter()
			.map(|p| (p.name.clone(), p.clone().into()))
			.collect()
	}

	pub fn exclusives(&self, pointers: &HashMap<String, Pointer>) -> Vec<Exclusives> {
		let mut exclusives = Vec::new();
		for exclusive in &self.exclusives {
			let Some(target) = &exclusive.target else {
				continue;
			};

			if let Some(pointer) = pointers.get(target) {
				exclusives.push(Exclusives::One(pointer.clone()));
				continue;
			}

			if target.starts_with('(') && target.ends_with(')') {
				// Handle multiple targets case
				let targets = target
					.trim_matches(|c| c == '(' || c == ')')
					.split(' ')
					.map(|t| {
						t.trim()
							.trim_start_matches('.')
							.trim_end_matches(',')
							.to_string()
					})
					.collect::<Vec<_>>();

				let mut target_pointers = vec![];

				for target in &targets {
					if let Some(pointer) = pointers.get(target) {
						target_pointers.push(pointer.clone());
					}
				}

				if target_pointers.is_empty() {
					continue;
				}

				if target_pointers.len() == 1 {
					exclusives.push(Exclusives::One(target_pointers[0].clone()));
				} else {
					exclusives.push(Exclusives::Many(target_pointers));
				}
			}
		}

		exclusives
	}

	pub fn backlinks(&self) -> Vec<Backlink> {
		let re = regex::Regex::new(r"\[is (.+)\]").unwrap();
		let mut backlinks = Vec::new();

		for backlink in &self.backlinks {
			let Some(target_id) = backlink.target_id else {
				continue;
			};

			let Some(matched_name) = re.captures(&backlink.name).and_then(|c| c.get(1)) else {
				continue;
			};

			let mut new_backlink = Backlink {
				cardinality: backlink.cardinality(),
				name: backlink.name.clone(),
				target_id,
				is_exclusive: backlink.is_exclusive.unwrap_or_default(),
				stub: Some(backlink.stub.clone()),
			};

			let Some((module_name, local_name)) = matched_name.as_str().split_once("::") else {
				backlinks.push(new_backlink);
				continue;
			};

			if module_name != "default" {
				backlinks.push(new_backlink);
				continue;
			}

			new_backlink.name = re
				.replace(&new_backlink.name, |_: &regex::Captures| {
					format!("[is {local_name}]")
				})
				.to_string();

			backlinks.push(new_backlink);
		}

		for backlink in &self.backlink_stubs {
			let Some(target_id) = backlink.target_id else {
				continue;
			};

			let Some(captures) = re.captures(&backlink.name) else {
				continue;
			};

			let Some(matched_name) = captures.get(1) else {
				continue;
			};

			let mut new_backlink = Backlink {
				cardinality: backlink.cardinality(),
				name: backlink.name.clone(),
				target_id,
				is_exclusive: backlink.is_exclusive,
				stub: None,
			};

			let Some((module_name, local_name)) = matched_name.as_str().split_once("::") else {
				backlinks.push(new_backlink);
				continue;
			};

			if module_name != "default" {
				backlinks.push(new_backlink);
				continue;
			}

			new_backlink.name = re
				.replace(&new_backlink.name, |_: &regex::Captures| {
					format!("[is {local_name}]")
				})
				.to_string();

			backlinks.push(new_backlink);
		}

		backlinks
	}
}

fn to_cardinality(cardinality: Option<&str>) -> Cardinality {
	let Some(cardinality) = cardinality else {
		return Cardinality::NoResult;
	};

	match cardinality {
		"AtMostOne" => Cardinality::AtMostOne,
		"One" => Cardinality::One,
		"Many" => Cardinality::Many,
		"AtLeastOne" => Cardinality::AtLeastOne,
		_ => Cardinality::NoResult,
	}
}

// Assuming Uuid, Cardinality, HashMap are in scope.
// use uuid::Uuid;
// use crate::Cardinality; // Or appropriate path
// use std::collections::HashMap;
// use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerKind {
	Link,
	Property,
}

bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct PointerFlags: u8 {
		const IS_EXCLUSIVE = 0b0000_0001;
		const IS_COMPUTED = 0b0000_0010;
		const IS_READONLY = 0b0000_0100;
		const HAS_DEFAULT = 0b0000_1000;
	}
}

#[derive(Debug, Clone)]
pub struct Pointer {
	pub card: Cardinality,
	pub kind: PointerKind,
	pub name: String,
	pub target_id: Uuid,
	pub flags: PointerFlags,
	pub pointers: Option<Vec<Pointer>>,
}

impl From<PointersSet> for Pointer {
	fn from(value: PointersSet) -> Self {
		let mut flags = PointerFlags::empty();

		if value.is_exclusive {
			flags.insert(PointerFlags::IS_EXCLUSIVE);
		}

		if value.is_computed.unwrap_or_default() {
			flags.insert(PointerFlags::IS_COMPUTED);
		}

		if value.is_readonly.unwrap_or_default() {
			flags.insert(PointerFlags::IS_READONLY);
		}

		if value.has_default {
			flags.insert(PointerFlags::HAS_DEFAULT);
		}

		Pointer {
			card: value.cardinality(),
			kind: match value.kind.as_str() {
				"link" => PointerKind::Link,
				"property" => PointerKind::Property,
				_ => panic!("Invalid pointer kind: {}", value.kind),
			},
			name: value.name,
			target_id: value.target_id.unwrap(),
			flags,
			pointers: Some(value.pointers.iter().map(|p| p.clone().into()).collect()),
		}
	}
}

impl From<PointersSetPointersSet> for Pointer {
	fn from(value: PointersSetPointersSet) -> Self {
		let mut flags = PointerFlags::empty();

		if value.is_computed.unwrap_or_default() {
			flags.insert(PointerFlags::IS_COMPUTED);
		}

		if value.is_readonly.unwrap_or_default() {
			flags.insert(PointerFlags::IS_READONLY);
		}

		Pointer {
			card: value.cardinality(),
			kind: match value.kind.as_str() {
				"link" => PointerKind::Link,
				"property" => PointerKind::Property,
				_ => panic!("Invalid pointer kind: {}", value.kind),
			},
			name: value.name,
			target_id: value.target_id.unwrap(),
			flags,
			pointers: None,
		}
	}
}

impl Pointer {
	pub fn is_link(&self) -> bool {
		self.kind == PointerKind::Link
	}

	pub fn is_property(&self) -> bool {
		self.kind == PointerKind::Property
	}

	pub fn is_exclusive(&self) -> bool {
		self.flags.contains(PointerFlags::IS_EXCLUSIVE)
	}

	pub fn is_computed(&self) -> bool {
		self.flags.contains(PointerFlags::IS_COMPUTED)
	}

	pub fn has_default(&self) -> bool {
		self.flags.contains(PointerFlags::HAS_DEFAULT)
	}

	pub fn is_readonly(&self) -> bool {
		self.flags.contains(PointerFlags::IS_READONLY)
	}
}

#[derive(Debug, Clone)]
pub struct Backlink {
	// Fields from Pointer without flags, kind, pointers
	pub cardinality: Cardinality,
	pub name: String,
	pub target_id: Uuid,

	// Specific to Backlink
	pub is_exclusive: bool,
	pub stub: Option<String>,
}

impl Backlink {
	pub fn is_stub(&self) -> bool {
		self.stub.is_none()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
	Object,
	Scalar,
	Array,
	Tuple,
	Range,
	MultiRange,
}

// Helper struct for fields like `bases: readonly { id: UUID }[]`
#[derive(Debug, Clone)]
pub struct IdRef {
	pub id: Uuid,
}

impl From<BasesSet> for IdRef {
	fn from(value: BasesSet) -> Self {
		IdRef { id: value.id }
	}
}

impl From<UnionOfSet> for IdRef {
	fn from(value: UnionOfSet) -> Self {
		IdRef { id: value.id }
	}
}

impl From<IntersectionOfSet> for IdRef {
	fn from(value: IntersectionOfSet) -> Self {
		IdRef { id: value.id }
	}
}

// Structs for each type kind. Note: 'kind' field is handled by the Type enum
// tag.
#[derive(Debug, Clone)]
pub struct ScalarType {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: bool,
	pub is_seq: bool,
	pub bases: Vec<IdRef>,
	pub material_id: Option<Uuid>,
	pub cast_type: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct EnumType {
	pub id: Uuid,
	pub name: String,
	pub enum_values: Vec<String>,
	pub bases: Vec<IdRef>,
}

#[derive(Clone, Debug)]
pub enum Exclusives {
	One(Pointer),
	Many(Vec<Pointer>),
}

#[derive(Debug, Clone)]
pub struct ObjectType {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: bool,
	pub bases: Vec<IdRef>,
	pub union_of: Vec<IdRef>,
	pub intersection_of: Vec<IdRef>,
	pub pointers: Vec<Pointer>,
	pub backlinks: Vec<Backlink>,
	pub exclusives: Vec<Exclusives>,
}

#[derive(Debug, Clone)]
pub struct ArrayType {
	pub id: Uuid,
	pub bases: Vec<IdRef>,
	pub name: String,
	pub array_element_id: Uuid,
	pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct TupleElementDef {
	pub name: String,
	pub target_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct TupleType {
	pub id: Uuid,
	pub name: String,
	pub tuple_elements: Vec<TupleElementDef>,
	pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct RangeType {
	pub id: Uuid,
	pub name: String,
	pub range_element_id: Uuid,
	pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct MultiRangeType {
	pub id: Uuid,
	pub name: String,
	pub multirange_element_id: Uuid,
	pub is_abstract: bool,
}

#[derive(Debug, Clone)]
pub struct BaseType {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: bool, // In TS, this is specified as 'false' for BaseType
}

#[derive(Debug, Clone)]
pub enum Type {
	Object(ObjectType),
	Scalar(ScalarType),
	Enum(EnumType),
	Array(ArrayType),
	Tuple(TupleType),
	Range(RangeType),
	MultiRange(MultiRangeType),
	Base(BaseType),
}

impl Type {
	pub fn id(&self) -> Uuid {
		match self {
			Type::Object(obj) => obj.id,
			Type::Scalar(scalar) => scalar.id,
			Type::Enum(enum_type) => enum_type.id,
			Type::Array(array_type) => array_type.id,
			Type::Tuple(tuple_type) => tuple_type.id,
			Type::Range(range_type) => range_type.id,
			Type::MultiRange(multi_range_type) => multi_range_type.id,
			Type::Base(base_type) => base_type.id,
		}
	}

	pub fn name(&self) -> &str {
		match self {
			Type::Object(obj) => &obj.name,
			Type::Scalar(scalar) => &scalar.name,
			Type::Enum(enum_type) => &enum_type.name,
			Type::Array(array_type) => &array_type.name,
			Type::Tuple(tuple_type) => &tuple_type.name,
			Type::Range(range_type) => &range_type.name,
			Type::MultiRange(multi_range_type) => &multi_range_type.name,
			Type::Base(base_type) => &base_type.name,
		}
	}

	pub fn is_primitive(&self) -> bool {
		matches!(
			self,
			Type::Scalar(_)
				| Type::Array(_)
				| Type::Tuple(_)
				| Type::Range(_)
				| Type::MultiRange(_)
		)
	}
}

pub type Types = IndexMap<Uuid, Type>;

fn map_fetched_types(fetched_types: &[TypesOutput]) -> Types {
	let mut types = IndexMap::new();

	for type_info in fetched_types {
		match type_info.kind.as_str() {
			"scalar" => {
				if let Some(enum_values) = &type_info.enum_values {
					let enum_type = EnumType {
						id: type_info.id,
						name: type_info.name.clone(),
						enum_values: enum_values.clone(),
						bases: type_info
							.bases
							.iter()
							.map(|base| base.clone().into())
							.collect(),
					};
					types.insert(type_info.id, Type::Enum(enum_type));
				} else {
					let scalar_type = ScalarType {
						id: type_info.id,
						name: type_info.name.clone(),
						is_abstract: type_info.is_abstract.unwrap_or_default(),
						is_seq: type_info.is_seq,
						bases: type_info
							.bases
							.iter()
							.map(|base| base.clone().into())
							.collect(),
						material_id: type_info.material_id,
						// TODO: doesn't seem useful in rust
						cast_type: None,
					};
					types.insert(type_info.id, Type::Scalar(scalar_type));
				}
			}
			"range" => {
				let range_type = RangeType {
					id: type_info.id,
					name: type_info.name.clone(),
					range_element_id: type_info.multirange_element_id.unwrap(),
					is_abstract: type_info.is_abstract.unwrap_or_default(),
				};
				types.insert(type_info.id, Type::Range(range_type));
			}
			"multirange" => {
				let multirange_type = MultiRangeType {
					id: type_info.id,
					name: type_info.name.clone(),
					multirange_element_id: type_info.multirange_element_id.unwrap(),
					is_abstract: type_info.is_abstract.unwrap_or_default(),
				};
				types.insert(type_info.id, Type::MultiRange(multirange_type));
			}
			"object" => {
				let pointers = type_info.pointers_map();
				let exclusives = type_info.exclusives(&pointers);
				let object_type = ObjectType {
					id: type_info.id,
					name: type_info.name.clone(),
					is_abstract: type_info.is_abstract.unwrap_or_default(),
					bases: type_info
						.bases
						.iter()
						.map(|base| base.clone().into())
						.collect(),
					union_of: type_info
						.union_of
						.iter()
						.map(|u| u.clone().into())
						.collect(),
					intersection_of: type_info
						.intersection_of
						.iter()
						.map(|i| i.clone().into())
						.collect(),
					pointers: type_info
						.pointers
						.iter()
						.map(|p| p.clone().into())
						.collect(),
					backlinks: type_info.backlinks(),
					exclusives,
				};
				types.insert(type_info.id, Type::Object(object_type));
			}
			_ => {}
		}
	}

	types
}

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
