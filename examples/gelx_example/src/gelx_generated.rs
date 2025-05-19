#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused)]
use ::gelx::exports as e;
#[derive(Clone, Debug, Copy, e::serde::Serialize, e::serde::Deserialize)]
#[cfg_attr(
    feature = "ssr",
    derive(
        e::gel_derive::Queryable,
        e::strum::AsRefStr,
        e::strum::Display,
        e::strum::EnumString,
        e::strum::EnumIs,
        e::strum::FromRepr,
        e::strum::IntoStaticStr
    )
)]
pub enum AccountProvider {
    Github,
}
#[derive(Clone, Debug, Copy, e::serde::Serialize, e::serde::Deserialize)]
#[cfg_attr(
    feature = "ssr",
    derive(
        e::gel_derive::Queryable,
        e::strum::AsRefStr,
        e::strum::Display,
        e::strum::EnumString,
        e::strum::EnumIs,
        e::strum::FromRepr,
        e::strum::IntoStaticStr
    )
)]
pub enum RelationshipType {
    Follow,
    Block,
    Mute,
}
#[derive(Clone, Debug, Copy, e::serde::Serialize, e::serde::Deserialize)]
#[cfg_attr(
    feature = "ssr",
    derive(
        e::gel_derive::Queryable,
        e::strum::AsRefStr,
        e::strum::Display,
        e::strum::EnumString,
        e::strum::EnumIs,
        e::strum::FromRepr,
        e::strum::IntoStaticStr
    )
)]
pub enum Role {
    None,
    Editor,
    Moderator,
    Admin,
    Owner,
}
pub mod additional {
    use super::*;
    #[derive(Clone, Debug, Copy, e::serde::Serialize, e::serde::Deserialize)]
    #[cfg_attr(
        feature = "ssr",
        derive(
            e::gel_derive::Queryable,
            e::strum::AsRefStr,
            e::strum::Display,
            e::strum::EnumString,
            e::strum::EnumIs,
            e::strum::FromRepr,
            e::strum::IntoStaticStr
        )
    )]
    pub enum Awesomeness {
        Very,
        Somewhat,
        NotReally,
    }
    #[derive(Clone, Debug, Copy, e::serde::Serialize, e::serde::Deserialize)]
    #[cfg_attr(
        feature = "ssr",
        derive(
            e::gel_derive::Queryable,
            e::strum::AsRefStr,
            e::strum::Display,
            e::strum::EnumString,
            e::strum::EnumIs,
            e::strum::FromRepr,
            e::strum::IntoStaticStr
        )
    )]
    pub enum Smartness {
        #[serde(rename = "low")]
        #[cfg_attr(feature = "ssr", strum(serialize = "low"))]
        Low,
        #[serde(rename = "mid")]
        #[cfg_attr(feature = "ssr", strum(serialize = "mid"))]
        Mid,
        #[serde(rename = "genius")]
        #[cfg_attr(feature = "ssr", strum(serialize = "genius"))]
        Genius,
    }
}
pub mod select_user {
    use ::gelx::exports as e;
    /// Execute the desired query.
    #[cfg(feature = "ssr")]
    pub async fn query(
        client: &e::gel_tokio::Client,
        props: &Input,
    ) -> Result<Option<Output>, e::gel_errors::Error> {
        client.query_single(QUERY, props).await
    }
    /// Compose the query as part of a larger transaction.
    #[cfg(feature = "ssr")]
    pub async fn transaction(
        conn: &mut e::gel_tokio::Transaction,
        props: &Input,
    ) -> Result<Option<Output>, e::gel_errors::Error> {
        conn.query_single(QUERY, props).await
    }
    #[derive(Clone, Debug, e::serde::Serialize, e::serde::Deserialize)]
    #[cfg_attr(
        feature = "ssr",
        derive(e::typed_builder::TypedBuilder, e::gel_derive::Queryable)
    )]
    pub struct Input {
        #[cfg_attr(feature = "ssr", builder(setter(into)))]
        pub slug: String,
    }
    impl gel_protocol::query_arg::QueryArgs for Input {
        fn encode(
            &self,
            encoder: &mut gel_protocol::query_arg::Encoder,
        ) -> Result<(), e::gel_errors::Error> {
            let map = e::gel_protocol::named_args! {
                "slug" => self.slug.clone(),
            };
            map.encode(encoder)
        }
    }
    #[derive(Clone, Debug, e::serde::Serialize, e::serde::Deserialize)]
    #[cfg_attr(feature = "ssr", derive(e::gel_derive::Queryable))]
    pub struct Output {
        pub id: e::uuid::Uuid,
        pub name: Option<String>,
        pub bio: Option<String>,
        pub slug: String,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "select User {\n\tid,\n  name,\n  bio,\n  slug,\n} filter .slug = <str>$slug;";
}
pub mod insert_user {
    use ::gelx::exports as e;
    /// Execute the desired query.
    #[cfg(feature = "ssr")]
    pub async fn query(
        client: &e::gel_tokio::Client,
        props: &Input,
    ) -> Result<Output, e::gel_errors::Error> {
        client.query_required_single(QUERY, props).await
    }
    /// Compose the query as part of a larger transaction.
    #[cfg(feature = "ssr")]
    pub async fn transaction(
        conn: &mut e::gel_tokio::Transaction,
        props: &Input,
    ) -> Result<Output, e::gel_errors::Error> {
        conn.query_required_single(QUERY, props).await
    }
    #[derive(Clone, Debug, e::serde::Serialize, e::serde::Deserialize)]
    #[cfg_attr(
        feature = "ssr",
        derive(e::typed_builder::TypedBuilder, e::gel_derive::Queryable)
    )]
    pub struct Input {
        #[cfg_attr(feature = "ssr", builder(setter(into)))]
        pub name: String,
        #[cfg_attr(feature = "ssr", builder(setter(into)))]
        pub bio: String,
        #[cfg_attr(feature = "ssr", builder(setter(into)))]
        pub slug: String,
    }
    impl gel_protocol::query_arg::QueryArgs for Input {
        fn encode(
            &self,
            encoder: &mut gel_protocol::query_arg::Encoder,
        ) -> Result<(), e::gel_errors::Error> {
            let map = e::gel_protocol::named_args! {
                "name" => self.name.clone(), "bio" => self.bio.clone(), "slug" => self
                .slug.clone(),
            };
            map.encode(encoder)
        }
    }
    #[derive(Clone, Debug, e::serde::Serialize, e::serde::Deserialize)]
    #[cfg_attr(feature = "ssr", derive(e::gel_derive::Queryable))]
    pub struct Output {
        pub id: e::uuid::Uuid,
        pub name: Option<String>,
        pub bio: Option<String>,
        pub slug: String,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "with NewUser := (insert User {\n  name := <str>$name,\n  bio := <str>$bio,\n  slug := <str>$slug,\n})\nselect NewUser {\n  id,\n  name,\n  bio,\n  slug,\n};\n";
}
pub mod remove_user {
    use ::gelx::exports as e;
    /// Execute the desired query.
    #[cfg(feature = "ssr")]
    pub async fn query(
        client: &e::gel_tokio::Client,
        props: &Input,
    ) -> Result<Option<Output>, e::gel_errors::Error> {
        client.query_single(QUERY, props).await
    }
    /// Compose the query as part of a larger transaction.
    #[cfg(feature = "ssr")]
    pub async fn transaction(
        conn: &mut e::gel_tokio::Transaction,
        props: &Input,
    ) -> Result<Option<Output>, e::gel_errors::Error> {
        conn.query_single(QUERY, props).await
    }
    #[derive(Clone, Debug, e::serde::Serialize, e::serde::Deserialize)]
    #[cfg_attr(
        feature = "ssr",
        derive(e::typed_builder::TypedBuilder, e::gel_derive::Queryable)
    )]
    pub struct Input {
        #[cfg_attr(feature = "ssr", builder(setter(into)))]
        pub id: e::uuid::Uuid,
    }
    impl gel_protocol::query_arg::QueryArgs for Input {
        fn encode(
            &self,
            encoder: &mut gel_protocol::query_arg::Encoder,
        ) -> Result<(), e::gel_errors::Error> {
            let map = e::gel_protocol::named_args! {
                "id" => self.id,
            };
            map.encode(encoder)
        }
    }
    #[derive(Clone, Debug, e::serde::Serialize, e::serde::Deserialize)]
    #[cfg_attr(feature = "ssr", derive(e::gel_derive::Queryable))]
    pub struct Output {
        pub id: e::uuid::Uuid,
    }
    /// The original query string provided to the macro. Can be reused in your codebase.
    pub const QUERY: &str = "delete User filter .id = <uuid>$id;\n";
}
