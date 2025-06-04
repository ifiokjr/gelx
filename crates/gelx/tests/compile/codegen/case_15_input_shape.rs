pub mod example {
    use ::gelx::exports as __g;
    /// Execute the desired query.
    pub async fn query(
        client: &__g::gel_tokio::Client,
    ) -> ::core::result::Result<Output, __g::gel_errors::Error> {
        client.query_required_single(QUERY, &()).await
    }
    /// Compose the query as part of a larger transaction.
    pub async fn transaction(
        conn: &mut __g::gel_tokio::Transaction,
    ) -> ::core::result::Result<Output, __g::gel_errors::Error> {
        conn.query_required_single(QUERY, &()).await
    }
    pub type Input = ();
    #[derive(
        Debug,
        Clone,
        Copy,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable,
        __g::strum::AsRefStr,
        __g::strum::Display,
        __g::strum::EnumString,
        __g::strum::EnumIs,
        __g::strum::FromRepr,
        __g::strum::IntoStaticStr
    )]
    #[strum(crate = "__g::strum")]
    pub enum DefaultRelationshipType {
        Follow,
        Block,
        Mute,
    }
    impl From<DefaultRelationshipType> for __g::gel_protocol::value::Value {
        fn from(value: DefaultRelationshipType) -> Self {
            __g::gel_protocol::value::Value::Enum(value.as_ref().into())
        }
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    pub struct Output {
        pub my_string: DefaultRelationshipType,
        pub my_number: i64,
        pub several_numbers: Vec<i64>,
        pub array: Vec<i64>,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "select { my_string := RelationshipType.Follow, my_number := 42, several_numbers := {1, 2, 3}, array := [1, 2, 3] };";
}
fn main() {}
