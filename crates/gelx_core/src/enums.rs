use check_keyword::CheckKeyword;
use gel_derive::Queryable;
use gel_tokio::Client;
use heck::ToPascalCase;
use indexmap::indexmap;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::EXPORTS_IDENT;
use crate::FeatureName;
use crate::GelxCoreResult;
use crate::GelxMetadata;
use crate::TYPES_QUERY;
use crate::create_gel_config;

/// Execute the types query to get the types of the current database.
async fn types_query(client: &Client) -> GelxCoreResult<Vec<TypesOutput>> {
	let result = Box::pin(client.query(TYPES_QUERY, &())).await?;

	Ok(result)
}

/// Generate the custom enums.
pub async fn generate_enums(
	metadata: &GelxMetadata,
	is_macro: bool,
) -> GelxCoreResult<TokenStream> {
	let mut tokens_map = indexmap! {
		"default".to_string() => quote!(use ::gelx::exports as #EXPORTS_IDENT;),
	};
	let mut tokens: TokenStream = TokenStream::new();
	let config = create_gel_config(
		metadata.gel_config_path.as_deref(),
		metadata.gel_instance.as_deref(),
		metadata.gel_branch.as_deref(),
	)?;
	let client = Client::new(&config);
	let fetched_types = types_query(&client).await?;

	for type_info in &fetched_types {
		let Some(enum_values) = &type_info.enum_values else {
			continue;
		};

		if type_info.name.starts_with("std::")
			|| type_info.name.starts_with("sys::")
			|| type_info.name.starts_with("cfg::")
			|| type_info.name.starts_with("schema::")
		{
			continue;
		}

		let Some((module_name, local_name)) = type_info.name.split_once("::") else {
			continue;
		};

		let module_tokens = tokens_map
			.entry(module_name.into_safe())
			.or_insert_with(|| {
				if module_name == "default" {
					quote!(use ::gelx::exports as #EXPORTS_IDENT;)
				} else {
					quote! { use super::*; }
				}
			});

		let enum_tokens = generate_enum(metadata, enum_values, local_name, is_macro);

		module_tokens.extend(enum_tokens);
	}

	for (module_name, module_tokens) in tokens_map {
		if module_name == "default" {
			tokens.extend(module_tokens);
		} else {
			let module_name_token = format_ident!("{}", module_name);
			tokens.extend(quote! {
				pub mod #module_name_token {
					#module_tokens
				}
			});
		}
	}

	Ok(tokens)
}

pub(crate) fn generate_enum(
	metadata: &GelxMetadata,
	enum_values: &[String],
	local_name: &str,
	is_macro: bool,
) -> TokenStream {
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

	let enum_derive = metadata.features.get_enum_derive_features(is_macro);
	let enum_tokens = quote! {
		#enum_derive
		pub enum #pascal_local_name {
			#(#enum_values_tokens),*
		}
	};

	enum_tokens
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputBasesSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputUnionOfSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputIntersectionOfSet {
	pub id: Uuid,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputPointersSetPointersSet {
	pub card: Option<String>,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_computed: Option<bool>,
	pub is_readonly: Option<bool>,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputPointersSet {
	pub card: Option<String>,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: bool,
	pub is_computed: Option<bool>,
	pub is_readonly: Option<bool>,
	pub has_default: bool,
	pub pointers: Vec<OutputPointersSetPointersSet>,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputExclusivesSet {
	pub target: Option<String>,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputBacklinksSet {
	pub card: String,
	pub name: String,
	pub stub: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: Option<bool>,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputBacklinkStubsArray {
	pub card: String,
	pub name: String,
	pub target_id: Option<Uuid>,
	pub kind: String,
	pub is_exclusive: bool,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct OutputTupleElementsSet {
	pub target_id: Uuid,
	pub name: Option<String>,
}

#[derive(Clone, Debug, Queryable, Serialize, Deserialize)]
pub struct TypesOutput {
	pub id: Uuid,
	pub name: String,
	pub is_abstract: Option<bool>,
	pub kind: String,
	pub enum_values: Option<Vec<String>>,
	pub is_seq: bool,
	pub material_id: Option<Uuid>,
	pub bases: Vec<OutputBasesSet>,
	pub union_of: Vec<OutputUnionOfSet>,
	pub intersection_of: Vec<OutputIntersectionOfSet>,
	pub pointers: Vec<OutputPointersSet>,
	pub exclusives: Vec<OutputExclusivesSet>,
	pub backlinks: Vec<OutputBacklinksSet>,
	pub backlink_stubs: Vec<OutputBacklinkStubsArray>,
	pub array_element_id: Option<Uuid>,
	pub tuple_elements: Vec<OutputTupleElementsSet>,
	pub multirange_element_id: Option<Uuid>,
}
