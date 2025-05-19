pub mod example {
    use ::gelx::exports as e;
    /// Execute the desired query.
    #[cfg(feature = "query")]
    pub async fn query(
        client: &e::gel_tokio::Client,
    ) -> core::result::Result<Vec<Output>, e::gel_errors::Error> {
        client.query(QUERY, &()).await
    }
    /// Compose the query as part of a larger transaction.
    #[cfg(feature = "query")]
    pub async fn transaction(
        conn: &mut e::gel_tokio::Transaction,
    ) -> core::result::Result<Vec<Output>, e::gel_errors::Error> {
        conn.query(QUERY, &()).await
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
    pub enum DefaultAccountProvider {
        Github,
    }
    #[derive(
        Clone,
        Debug,
        e::serde::Serialize,
        e::serde::Deserialize,
        e::gel_derive::Queryable
    )]
    pub struct Output {
        pub provider: DefaultAccountProvider,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "select Account { provider }";
}
fn main() {}
