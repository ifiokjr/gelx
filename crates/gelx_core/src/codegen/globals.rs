use gel_protocol::value::Value;
use gel_tokio::Queryable;
use strum::AsRefStr;

/// Execute the desired query.
pub async fn query_globals(
	client: &gel_tokio::Client,
) -> ::core::result::Result<Vec<GlobalsOutput>, gel_errors::Error> {
	client.query(GLOBALS_QUERY, &()).await
}

#[derive(Debug, Clone, Copy, AsRefStr, Queryable)]
pub enum SchemaCardinality {
	One,
	Many,
}
impl From<SchemaCardinality> for Value {
	fn from(value: SchemaCardinality) -> Self {
		Value::Enum(value.as_ref().into())
	}
}

#[derive(Debug, Clone, Queryable)]
#[allow(clippy::struct_excessive_bools)]
pub struct GlobalsTarget {
	pub id: uuid::Uuid,
	pub name: String,
	pub is_from_alias: Option<bool>,
}
#[derive(Debug, Clone, Queryable)]
pub struct GlobalsOutput {
	pub id: uuid::Uuid,
	pub name: String,
	pub cardinality: Option<SchemaCardinality>,
	pub target: Option<GlobalsTarget>,
}

pub const GLOBALS_QUERY: &str =
	"select schema::Global {id, name, cardinality, target: {id, name, is_from_alias}}";
