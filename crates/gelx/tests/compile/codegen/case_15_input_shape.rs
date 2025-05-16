fn main() {
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
        #[derive(Clone, Debug)]
        #[cfg_attr(feature = "query", derive(e::gel_derive::Queryable))]
        #[cfg_attr(
            feature = "serde",
            derive(e::serde::Serialize, e::serde::Deserialize)
        )]
        pub struct Output {
            pub my_string: String,
            pub my_number: i64,
            pub several_numbers: Vec<i64>,
            pub array: Vec<i64>,
        }
        /// The original query string provided to the macro. Can be reused in your codebase.
        pub const QUERY: &str = "select { my_string := RelationshipType.Follow, my_number := 42, several_numbers := {1, 2, 3}, array := [1, 2, 3] };";
    }
}
