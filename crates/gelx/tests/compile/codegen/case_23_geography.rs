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
    pub type Output = __g::Geography;
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "select <ext::postgis::geography>ext::postgis::makepoint(1.0, 1.0)";
}
fn main() {}
