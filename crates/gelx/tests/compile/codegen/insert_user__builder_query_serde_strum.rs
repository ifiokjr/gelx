pub mod example {
    use ::gelx::exports as __g;
    /// Execute the desired query.
    pub async fn query(
        client: &__g::gel_tokio::Client,
        props: &Input,
    ) -> ::core::result::Result<Output, __g::gel_errors::Error> {
        client.query_required_single(QUERY, props).await
    }
    /// Compose the query as part of a larger transaction.
    pub async fn transaction(
        conn: &mut __g::gel_tokio::Transaction,
        props: &Input,
    ) -> ::core::result::Result<Output, __g::gel_errors::Error> {
        conn.query_required_single(QUERY, props).await
    }
    #[derive(
        Debug,
        Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::typed_builder::TypedBuilder,
        __g::gel_derive::Queryable
    )]
    pub struct Input {
        #[builder(setter(into))]
        pub name: String,
        #[builder(setter(into))]
        pub bio: String,
        #[builder(setter(into))]
        pub slug: String,
    }
    impl __g::gel_protocol::query_arg::QueryArgs for Input {
        fn encode(
            &self,
            encoder: &mut __g::gel_protocol::query_arg::Encoder,
        ) -> core::result::Result<(), __g::gel_errors::Error> {
            let map = __g::gel_protocol::named_args! {
                "name" => self.name.clone(), "bio" => self.bio.clone(), "slug" => self
                .slug.clone(),
            };
            map.encode(encoder)
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
        pub id: __g::uuid::Uuid,
        pub name: Option<String>,
        pub bio: Option<String>,
        pub slug: String,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "with NewUser := (insert User {\n  name := <str>$name,\n  bio := <str>$bio,\n  slug := <str>$slug,\n})\nselect NewUser {\n  id,\n  name,\n  bio,\n  slug,\n};\n";
}
fn main() {}
