use std::path::Path;

use gel_derive::Queryable;
use gel_errors::Error as GelError;
use gel_tokio::Builder;
use gel_tokio::Client;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::EXPORTS_IDENT;
use crate::Result;
use crate::TYPES_QUERY;
use crate::create_gel_config;
use crate::utils::FeatureAliases;

/// Execute the types query to get the types of the current database.
async fn types_query(client: &Client) -> Result<Vec<TypesOutput>> {
	let result = Box::pin(client.query(TYPES_QUERY, &())).await?;

	Ok(result)
}

/// Helper function to get Rust type (very basic for now)
/// TODO: Expand this significantly based on schema type_info and target_id
/// lookups
fn get_rust_type(
	gel_type_name: &str,
	target_id: Option<Uuid>,
	is_optional: bool,
	is_multi: bool,
	types_map: &std::collections::HashMap<Uuid, &TypesOutput>, // To look up target types
) -> TokenStream {
	let base_type_str = if let Some(tid) = target_id {
		types_map
			.get(&tid)
			.map_or("serde_json::Value".to_string(), |t| {
				t.name.replace("::", "__").replace("|", "_or_")
			})
	} else {
		// Map scalar types more accurately later
		match gel_type_name {
			"std::str" => "String".to_string(),
			"std::bool" => "bool".to_string(),
			"std::int16" => "i16".to_string(),
			"std::int32" => "i32".to_string(),
			"std::int64" => "i64".to_string(),
			"std::float32" => "f32".to_string(),
			"std::float64" => "f64".to_string(),
			"std::bigint" => "i64".to_string(), // Or use a bigint crate
			"std::uuid" => "Uuid".to_string(),
			"std::datetime" => "String".to_string(), // Or a datetime crate type
			"std::json" => "serde_json::Value".to_string(),
			_ => "serde_json::Value".to_string(), // Fallback for unknown scalars or complex types
		}
	};

	let base_type = format_ident!("{}", base_type_str);

	if is_multi {
		if is_optional {
			quote! { Option<Vec<#base_type>> }
		} else {
			quote! { Vec<#base_type> }
		}
	} else if is_optional {
		quote! { Option<#base_type> }
	} else {
		quote! { #base_type }
	}
}

/// Generate the types of the current database.
pub async fn generate_schema_token_stream<P: AsRef<Path>>(
	config_path: Option<P>,
) -> Result<TokenStream> {
	let mut tokens: TokenStream = TokenStream::new();
	let config = create_gel_config(config_path)?;
	let client = Client::new(&config);
	let fetched_types = types_query(&client).await?;

	// println!("TYPES: {:#?}", fetched_types);

	// Create a map for quick lookup of types by ID
	let types_map: std::collections::HashMap<Uuid, &TypesOutput> =
		fetched_types.iter().map(|t| (t.id, t)).collect();

	let feature_aliases = FeatureAliases::default(); // Assuming default aliases for now

	for type_info in &fetched_types {
		// Skip abstract types for now, or handle them differently (e.g. as traits)
		if type_info.is_abstract.unwrap_or(false) && type_info.kind != "scalar" {
			// Allow abstract scalars like Uuid if they are not enums
			if type_info.kind == "scalar" && type_info.enum_values.is_none() {
				// Potentially generate a type alias for abstract scalars if
				// needed For now, we skip direct struct generation for
				// abstract non-enum scalars
			} else {
				continue;
			}
		}

		// Sanitize type name for Rust
		let name_str = type_info.name.replace("::", "__").replace("|", "_or_");
		let struct_name = format_ident!("{}", name_str);
		let derives = feature_aliases.get_derive_features(false); // false for output structs
		let serde_rename_all = feature_aliases.wrap_annotation(
			crate::utils::FeatureName::Serde,
			&quote!(serde(rename_all = "camelCase")),
		);

		if type_info.kind == "scalar" && type_info.enum_values.is_some() {
			// Generate Enum
			let enum_values = type_info.enum_values.as_ref().unwrap();
			let variants = enum_values.iter().map(|v| {
				let variant_name = format_ident!("{}", v.replace(" ", "_")); // Sanitize enum variant names
				quote! { #variant_name }
			});

			let enum_def = quote! {
				#derives
				#serde_rename_all
				pub enum #struct_name {
					#(#variants),*
				}
			};
			tokens.extend(enum_def);
			continue; // Move to the next type
		}

		let mut fields = TokenStream::new();

		// Process pointers
		for pointer in &type_info.pointers {
			let field_name_str = &pointer.name.to_snake_case();
			let field_name = format_ident!("{}", field_name_str);
			// Cardinality: One, AtMostOne, Many, AtLeastOne
			let is_optional = pointer.card == Some("AtMostOne".to_string())
				|| pointer.card == Some("Many".to_string());
			let is_multi = pointer.card == Some("Many".to_string())
				|| pointer.card == Some("AtLeastOne".to_string());
			let field_type = get_rust_type(
				&pointer.kind,
				pointer.target_id,
				is_optional,
				is_multi,
				&types_map,
			);
			let pointer_name = &pointer.name;
			let serde_rename = feature_aliases.wrap_annotation(
				crate::utils::FeatureName::Serde,
				&quote!(serde(rename = #pointer_name)),
			);

			fields.extend(quote! {
				#serde_rename
				pub #field_name: #field_type,
			});

			// Handle link properties if any (nested fields)
			for link_prop in &pointer.pointers {
				let link_prop_name_str = link_prop.name.to_snake_case();
				let link_prop_name = format_ident!("{}", link_prop_name_str);
				let lp_is_optional = link_prop.card == Some("AtMostOne".to_string())
					|| link_prop.card == Some("Many".to_string());
				let lp_is_multi = link_prop.card == Some("Many".to_string())
					|| link_prop.card == Some("AtLeastOne".to_string());
				let link_prop_type = get_rust_type(
					&link_prop.kind,
					link_prop.target_id,
					lp_is_optional,
					lp_is_multi,
					&types_map,
				);
				let serde_rename_link_prop = feature_aliases.wrap_annotation(
					crate::utils::FeatureName::Serde,
					&quote!(serde(rename = #link_prop_name)),
				);

				fields.extend(quote! {
					#serde_rename_link_prop
					pub #link_prop_name: #link_prop_type,
				});
			}
		}

		// Process backlinks
		for backlink in &type_info.backlinks {
			// Sanitize backlink name for Rust field name
			let name_sanitized = backlink
				.name
				.replace("<", "")
				.replace(">", "")
				.replace("[is ", "_is_")
				.replace("]", "");
			let field_name_str = name_sanitized.to_snake_case();
			let field_name = format_ident!("{}", field_name_str);
			// Cardinality: AtMostOne, Many
			let is_optional = backlink.card == "AtMostOne"; // Effectively, if not Many, it could be optional single
			let is_multi = backlink.card == "Many";
			let field_type = get_rust_type(
				&backlink.kind,
				backlink.target_id,
				is_optional,
				is_multi,
				&types_map,
			);
			let backlink_name = &backlink.name;
			let serde_rename = feature_aliases.wrap_annotation(
				crate::utils::FeatureName::Serde,
				&quote!(serde(rename = #backlink_name)),
			);

			fields.extend(quote! {
				#serde_rename
				pub #field_name: #field_type,
			});
		}

		// Handle array types
		if type_info.kind == "array" {
			if let Some(element_id) = type_info.array_element_id {
				let element_type =
					get_rust_type("array_element", Some(element_id), false, false, &types_map);
				// Arrays are inherently Vec<T>
				let array_def = quote! {
					#derives
					#serde_rename_all
					pub struct #struct_name {
						pub inner: Vec<#element_type>,
					}
				};
				tokens.extend(array_def);
				continue;
			}
		}

		// Handle tuple types
		if type_info.kind == "tuple" && !type_info.tuple_elements.is_empty() {
			let mut tuple_fields = TokenStream::new();
			for (idx, element) in type_info.tuple_elements.iter().enumerate() {
				let element_type = get_rust_type(
					"tuple_element",
					Some(element.target_id),
					false,
					false,
					&types_map,
				);
				// Tuple elements in Rust are positional, names are for Gel schema
				let field_ident = format_ident!("_{}", idx);
				tuple_fields.extend(quote! { pub #element_type, });
			}
			let tuple_struct_def = quote! {
				#derives
				// For named tuple structs, derive can be applied directly.
				// For actual tuple ( unnamed ), derive might need custom handling or newtype pattern.
				// Using struct with named fields for simplicity for now.
				// pub struct #struct_name ( #tuple_fields ); // This would be a true tuple struct
				// Let's try a struct with numbered fields for now
			};
			// This is tricky, a proper tuple struct `pub struct Name(Type1, Type2);`
			// or a struct with named fields `pub struct Name { _0: Type1, _1: Type2 }`
			// For now, let's generate a struct with named fields `_0`, `_1`, etc.
			// This aligns better if we want to add serde attributes per field.
			let mut numbered_fields = TokenStream::new();
			for (idx, element) in type_info.tuple_elements.iter().enumerate() {
				let field_name = format_ident!("_{}", idx);
				let field_type = get_rust_type(
					"tuple_element",
					Some(element.target_id),
					false,
					false,
					&types_map,
				);
				// If gel provides names for tuple elements, we could use them.
				// let field_name = if let Some(name) = &element.name {
				//    format_ident!("{}", to_snake_case(name))
				// } else {
				//    format_ident!("_{}", idx)
				// };
				// let serde_rename_tuple_field = if let Some(name) = &element.name {
				//    feature_aliases.wrap_annotation(
				// 		crate::utils::FeatureName::Serde,
				// 		&quote!(serde(rename = #name)),
				// 	)
				// } else {
				//    quote!{}
				// };

				numbered_fields.extend(quote! {
					// #serde_rename_tuple_field
					pub #field_name: #field_type,
				});
			}

			tokens.extend(quote! {
				#derives
				#serde_rename_all
				pub struct #struct_name {
					#numbered_fields
				}
			});

			continue;
		}

		if type_info.kind == "object"
			|| (type_info.kind == "scalar" && type_info.enum_values.is_none())
		{
			let struct_def = quote! {
				#derives
				#serde_rename_all
				pub struct #struct_name {
					#fields
				}
			};
			tokens.extend(struct_def);
		}
	}

	Ok(tokens)
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

#[cfg(test)]
mod tests {

	use super::*;

	#[tokio::test]
	async fn can_generate_types() -> Result<()> {
		let tokens = generate_schema_token_stream(Option::<&str>::None).await?;
		println!("{}", tokens);
		// A simple check to see if something is generated
		assert!(!tokens.is_empty(), "Generated tokens should not be empty");

		// You can write the output to a file to inspect it:
		// use std::io::Write;
		// let formatted =
		// crate::utils::rustfmt(&tokens.to_string()).await.unwrap_or_else(|_|
		// tokens.to_string()); let mut file =
		// std::fs::File::create("generated_schema.rs").unwrap();
		// file.write_all(formatted.as_bytes()).unwrap();

		Ok(())
	}
}
