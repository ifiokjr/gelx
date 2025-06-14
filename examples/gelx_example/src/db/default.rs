//! This file is generated by `gelx generate`.
//! It is not intended for manual editing.
//! To update it, run `gelx generate`.
#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused)]
#![allow(unused_qualifications)]
#![allow(clippy::all)]
use super::*;
mod account {
    use super::*;
}
#[derive(
    ::std::fmt::Debug,
    ::core::clone::Clone,
    ::core::marker::Copy,
    __g::strum::AsRefStr,
    __g::strum::Display,
    __g::strum::EnumString,
    __g::strum::EnumIs,
    __g::strum::FromRepr,
    __g::strum::IntoStaticStr
)]
#[cfg_attr(
    feature = "with_serde",
    derive(__g::serde::Serialize, __g::serde::Deserialize)
)]
#[cfg_attr(feature = "with_query", derive(__g::gel_derive::Queryable))]
#[cfg_attr(feature = "with_query", gel(crate_path = __g::gel_protocol))]
#[strum(crate = "__g::strum")]
pub enum AccountProvider {
    Github,
}
impl ::core::convert::From<AccountProvider> for __g::gel_protocol::value::Value {
    fn from(value: AccountProvider) -> Self {
        __g::gel_protocol::value::Value::Enum(value.as_ref().into())
    }
}
mod email {
    use super::*;
}
mod location {
    use super::*;
}
#[derive(::std::fmt::Debug, ::core::clone::Clone)]
#[cfg_attr(
    feature = "with_serde",
    derive(__g::serde::Serialize, __g::serde::Deserialize)
)]
pub struct Position(pub i32);
#[cfg(feature = "with_query")]
impl __g::gel_protocol::queryable::Queryable for Position {
    type Args = <i32 as __g::gel_protocol::queryable::Queryable>::Args;
    fn decode(
        decoder: &__g::gel_protocol::queryable::Decoder,
        args: &Self::Args,
        buf: &[u8],
    ) -> Result<Self, __g::gel_protocol::errors::DecodeError> {
        Ok(Self(i32::decode(decoder, args, buf)?))
    }
    fn check_descriptor(
        ctx: &__g::gel_protocol::queryable::DescriptorContext,
        type_pos: __g::gel_protocol::descriptors::TypePos,
    ) -> Result<Self::Args, __g::gel_protocol::queryable::DescriptorMismatch> {
        __g::check_scalar(
            ctx,
            type_pos,
            __g::uuid::Uuid::from_bytes([
                198u8, 101u8, 10u8, 120u8, 66u8, 209u8, 17u8, 240u8, 175u8, 163u8, 31u8,
                171u8, 222u8, 52u8, 143u8, 173u8,
            ]),
            "default::Position",
        )?;
        Ok(())
    }
}
impl ::core::convert::From<Position> for __g::gel_protocol::value::Value {
    fn from(value: Position) -> Self {
        value.0.into()
    }
}
impl ::core::convert::From<Position> for i32 {
    fn from(value: Position) -> Self {
        value.0
    }
}
impl ::core::convert::From<i32> for Position {
    fn from(value: i32) -> Self {
        Position(value)
    }
}
impl ::std::ops::Deref for Position {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::std::ops::DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
mod project {
    use super::*;
}
mod relationship {
    use super::*;
}
#[derive(
    ::std::fmt::Debug,
    ::core::clone::Clone,
    ::core::marker::Copy,
    __g::strum::AsRefStr,
    __g::strum::Display,
    __g::strum::EnumString,
    __g::strum::EnumIs,
    __g::strum::FromRepr,
    __g::strum::IntoStaticStr
)]
#[cfg_attr(
    feature = "with_serde",
    derive(__g::serde::Serialize, __g::serde::Deserialize)
)]
#[cfg_attr(feature = "with_query", derive(__g::gel_derive::Queryable))]
#[cfg_attr(feature = "with_query", gel(crate_path = __g::gel_protocol))]
#[strum(crate = "__g::strum")]
pub enum RelationshipType {
    Follow,
    Block,
    Mute,
}
impl ::core::convert::From<RelationshipType> for __g::gel_protocol::value::Value {
    fn from(value: RelationshipType) -> Self {
        __g::gel_protocol::value::Value::Enum(value.as_ref().into())
    }
}
#[derive(
    ::std::fmt::Debug,
    ::core::clone::Clone,
    ::core::marker::Copy,
    __g::strum::AsRefStr,
    __g::strum::Display,
    __g::strum::EnumString,
    __g::strum::EnumIs,
    __g::strum::FromRepr,
    __g::strum::IntoStaticStr
)]
#[cfg_attr(
    feature = "with_serde",
    derive(__g::serde::Serialize, __g::serde::Deserialize)
)]
#[cfg_attr(feature = "with_query", derive(__g::gel_derive::Queryable))]
#[cfg_attr(feature = "with_query", gel(crate_path = __g::gel_protocol))]
#[strum(crate = "__g::strum")]
pub enum Role {
    None,
    Editor,
    Moderator,
    Admin,
    Owner,
}
impl ::core::convert::From<Role> for __g::gel_protocol::value::Value {
    fn from(value: Role) -> Self {
        __g::gel_protocol::value::Value::Enum(value.as_ref().into())
    }
}
mod simple {
    use super::*;
}
mod team {
    use super::*;
}
mod user {
    use super::*;
}
mod wallet {
    use super::*;
}
