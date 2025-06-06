use check_keyword::CheckKeyword;
use gel_tokio::Client;
use heck::ToPascalCase;
use indexmap::IndexMap;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::LitByte;
use uuid::Uuid;

use super::*;
use crate::FeatureName;
use crate::GelxCoreResult;
use crate::GelxMetadata;
use crate::maybe_uuid_to_token_name;

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

pub(crate) fn generate_globals(metadata: &GelxMetadata, globals: &[GlobalsOutput]) -> TokenStream {
	let mut tokens = TokenStream::new();
	let exports_ident = metadata.exports_alias_ident();
	let (fields, setters): (Vec<_>, Vec<_>) = globals
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
		let derive_macro_paths = metadata.struct_derive_macro_paths();
		let struct_derive_tokens = metadata.features.get_struct_derive_features(
			&exports_ident,
			&derive_macro_paths,
			true,
			false,
		);
		let typed_builder_annotation = metadata.features.wrap_annotation(
			FeatureName::Builder,
			&quote!(builder(field_defaults(
				default,
				setter(into, strip_option(fallback_suffix = "_opt"))
			))),
			false,
		);
		let queryable_annotation = metadata.features.annotate(FeatureName::Query, false);
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

	tokens
}
