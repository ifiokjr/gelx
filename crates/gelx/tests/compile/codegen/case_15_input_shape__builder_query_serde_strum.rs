pub mod example {
    use ::gelx::exports as e;
    /// Execute the desired query.
    #[cfg(feature = "query")]
    pub async fn query(
        client: &e::gel_tokio::Client,
    ) -> core::result::Result<Output, e::gel_errors::Error> {
        client.query_required_single(QUERY, &()).await
    }
    /// Compose the query as part of a larger transaction.
    #[cfg(feature = "query")]
    pub async fn transaction(
        conn: &mut e::gel_tokio::Transaction,
    ) -> core::result::Result<Output, e::gel_errors::Error> {
        conn.query_required_single(QUERY, &()).await
    }
    pub type Input = ();
    #[derive(
        Clone,
        Debug,
        Copy,
        e::serde::Serialize,
        e::serde::Deserialize,
        e::gel_derive::Queryable,
        e::strum::AsRefStr,
        e::strum::Display,
        e::strum::EnumString,
        e::strum::EnumIs,
        e::strum::FromRepr,
        e::strum::IntoStaticStr
    )]
    pub enum DefaultRelationshipType {
        Follow,
        Block,
        Mute,
    }
    #[derive(
        Clone,
        Debug,
        e::serde::Serialize,
        e::serde::Deserialize,
        e::gel_derive::Queryable
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
