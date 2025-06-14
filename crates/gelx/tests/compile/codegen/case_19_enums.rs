pub mod example {
    use ::gelx::exports as __g;
    /// Execute the desired query.
    pub async fn query(
        client: &__g::gel_tokio::Client,
    ) -> ::core::result::Result<Vec<Output>, __g::gel_errors::Error> {
        client.query(QUERY, &()).await
    }
    /// Compose the query as part of a larger transaction.
    pub async fn transaction(
        conn: &mut __g::gel_tokio::Transaction,
    ) -> ::core::result::Result<Vec<Output>, __g::gel_errors::Error> {
        conn.query(QUERY, &()).await
    }
    pub type Input = ();
    #[derive(
        ::std::fmt::Debug,
        ::core::clone::Clone,
        ::core::marker::Copy,
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
    #[gel(crate_path = __g::gel_protocol)]
    #[strum(crate = "__g::strum")]
    pub enum DefaultAccountProvider {
        Github,
    }
    impl ::core::convert::From<DefaultAccountProvider>
    for __g::gel_protocol::value::Value {
        fn from(value: DefaultAccountProvider) -> Self {
            __g::gel_protocol::value::Value::Enum(value.as_ref().into())
        }
    }
    #[derive(
        ::std::fmt::Debug,
        ::core::clone::Clone,
        __g::serde::Serialize,
        __g::serde::Deserialize,
        __g::gel_derive::Queryable
    )]
    #[gel(crate_path = __g::gel_protocol)]
    pub struct Output {
        pub provider: DefaultAccountProvider,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "select Account { provider }";
}
fn main() {}
