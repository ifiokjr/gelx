use ::gelx::exports as __g;
#[derive(Debug, Clone, Copy, __g::serde::Serialize, __g::serde::Deserialize)]
#[cfg_attr(
    feature = "ssr",
    derive(
        __g::gel_derive::Queryable,
        __g::strum::AsRefStr,
        __g::strum::Display,
        __g::strum::EnumString,
        __g::strum::EnumIs,
        __g::strum::FromRepr,
        __g::strum::IntoStaticStr
    )
)]
pub enum AccountProvider {
    Github,
}
#[derive(Debug, Clone, Copy, __g::serde::Serialize, __g::serde::Deserialize)]
#[cfg_attr(
    feature = "ssr",
    derive(
        __g::gel_derive::Queryable,
        __g::strum::AsRefStr,
        __g::strum::Display,
        __g::strum::EnumString,
        __g::strum::EnumIs,
        __g::strum::FromRepr,
        __g::strum::IntoStaticStr
    )
)]
pub enum RelationshipType {
    Follow,
    Block,
    Mute,
}
#[derive(Debug, Clone, Copy, __g::serde::Serialize, __g::serde::Deserialize)]
#[cfg_attr(
    feature = "ssr",
    derive(
        __g::gel_derive::Queryable,
        __g::strum::AsRefStr,
        __g::strum::Display,
        __g::strum::EnumString,
        __g::strum::EnumIs,
        __g::strum::FromRepr,
        __g::strum::IntoStaticStr
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
    #[derive(Debug, Clone, Copy, __g::serde::Serialize, __g::serde::Deserialize)]
    #[cfg_attr(
        feature = "ssr",
        derive(
            __g::gel_derive::Queryable,
            __g::strum::AsRefStr,
            __g::strum::Display,
            __g::strum::EnumString,
            __g::strum::EnumIs,
            __g::strum::FromRepr,
            __g::strum::IntoStaticStr
        )
    )]
    pub enum Awesomeness {
        Very,
        Somewhat,
        NotReally,
    }
    #[derive(Debug, Clone, Copy, __g::serde::Serialize, __g::serde::Deserialize)]
    #[cfg_attr(
        feature = "ssr",
        derive(
            __g::gel_derive::Queryable,
            __g::strum::AsRefStr,
            __g::strum::Display,
            __g::strum::EnumString,
            __g::strum::EnumIs,
            __g::strum::FromRepr,
            __g::strum::IntoStaticStr
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
fn main() {}
