pub mod example {
    use ::gelx::exports as e;
    /// Execute the desired query.
    pub async fn query(
        client: &e::gel_tokio::Client,
    ) -> core::result::Result<Output, e::gel_errors::Error> {
        client.query_required_single(QUERY, &()).await
    }
    /// Compose the query as part of a larger transaction.
    pub async fn transaction(
        conn: &mut e::gel_tokio::Transaction,
    ) -> core::result::Result<Output, e::gel_errors::Error> {
        conn.query_required_single(QUERY, &()).await
    }
    pub type Input = ();
    pub type Output = e::BigIntAlias;
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "select 42n";
}
fn main() {}
