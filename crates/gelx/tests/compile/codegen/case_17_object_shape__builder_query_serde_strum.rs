pub mod example {
    use ::gelx::exports as e;
    /// Execute the desired query.
    pub async fn query(
        client: &e::gel_tokio::Client,
    ) -> ::core::result::Result<Vec<Output>, e::gel_errors::Error> {
        client.query(QUERY, &()).await
    }
    /// Compose the query as part of a larger transaction.
    pub async fn transaction(
        conn: &mut e::gel_tokio::Transaction,
    ) -> ::core::result::Result<Vec<Output>, e::gel_errors::Error> {
        conn.query(QUERY, &()).await
    }
    pub type Input = ();
    #[derive(
        Clone,
        Debug,
        e::serde::Serialize,
        e::serde::Deserialize,
        e::gel_derive::Queryable
    )]
    pub struct OutputWalletsSet {
        pub created_at: e::DateTimeAlias,
        pub id: e::uuid::Uuid,
        pub updated_at: e::DateTimeAlias,
        pub primary: bool,
        pub description: Option<String>,
        pub name: Option<String>,
        pub pubkey: String,
    }
    #[derive(
        Clone,
        Debug,
        e::serde::Serialize,
        e::serde::Deserialize,
        e::gel_derive::Queryable
    )]
    pub struct Output {
        pub slug: String,
        pub id: e::uuid::Uuid,
        pub created_at: e::DateTimeAlias,
        pub updated_at: e::DateTimeAlias,
        pub description: Option<String>,
        pub name: String,
        pub wallets: Vec<OutputWalletsSet>,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "select Team {**}";
}
fn main() {}
