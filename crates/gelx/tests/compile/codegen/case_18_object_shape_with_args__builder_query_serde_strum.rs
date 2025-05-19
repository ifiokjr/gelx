pub mod example {
    use ::gelx::exports as e;
    /// Execute the desired query.
    pub async fn query(
        client: &e::gel_tokio::Client,
        props: &Input,
    ) -> ::core::result::Result<Vec<Output>, e::gel_errors::Error> {
        client.query(QUERY, props).await
    }
    /// Compose the query as part of a larger transaction.
    pub async fn transaction(
        conn: &mut e::gel_tokio::Transaction,
        props: &Input,
    ) -> ::core::result::Result<Vec<Output>, e::gel_errors::Error> {
        conn.query(QUERY, props).await
    }
    #[derive(
        Clone,
        Debug,
        e::serde::Serialize,
        e::serde::Deserialize,
        e::typed_builder::TypedBuilder,
        e::gel_derive::Queryable
    )]
    pub struct Input {
        #[builder(setter(into))]
        pub starts_with: String,
        #[builder(setter(into))]
        pub ends_with: String,
    }
    impl e::gel_protocol::query_arg::QueryArgs for Input {
        fn encode(
            &self,
            encoder: &mut e::gel_protocol::query_arg::Encoder,
        ) -> core::result::Result<(), e::gel_errors::Error> {
            let map = e::gel_protocol::named_args! {
                "starts_with" => self.starts_with.clone(), "ends_with" => self.ends_with
                .clone(),
            };
            map.encode(encoder)
        }
    }
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
    pub const QUERY: &str = "select Team {**} filter .name like <str>$starts_with ++ '%' and .description like '%' ++ <str>$ends_with;";
}
fn main() {}
