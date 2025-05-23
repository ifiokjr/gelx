//! This file is generated by `gelx generate`.
//! It is not intended for manual editing.
//! To update it, run `gelx generate`.
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused)]
#![allow(unused_qualifications)]
#![allow(clippy::all)]
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
