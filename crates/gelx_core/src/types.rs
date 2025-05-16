use std::path::Path;

use gel_derive::Queryable;
use gel_errors::Error as GelError;
use gel_tokio::Builder;
use gel_tokio::Client;
use proc_macro2::TokenStream;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::Result;
use crate::TYPES_QUERY;
use crate::create_gel_config;

/// Execute the types query to get the types of the current database.
async fn types_query(client: &Client) -> Result<Vec<TypesOutput>> {
	let result = Box::pin(client.query(TYPES_QUERY, &())).await?;

	Ok(result)
}

/// Generate the types of the current database.
pub async fn generate_types<P: AsRef<Path>>(config_path: Option<P>) -> Result<TokenStream> {
	let mut tokens: TokenStream = TokenStream::new();
	let config = create_gel_config(config_path)?;
	let client = Client::new(&config);
	let types = types_query(&client).await?;

	println!("TYPES: {:#?}", types);

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
		let tokens = generate_types(Option::<&str>::None).await?;
		println!("TOKENS: {:#?}", tokens);

		Ok(())
	}
}
