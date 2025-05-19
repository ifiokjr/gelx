use gelx::exports as e;

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
	e::strum::IntoStaticStr,
)]
pub enum AccountProvider {
	Github,
}
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
	e::strum::IntoStaticStr,
)]
pub enum RelationshipType {
	Follow,
	Block,
	Mute,
}
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
	e::strum::IntoStaticStr,
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
		e::strum::IntoStaticStr,
	)]
	pub enum Awesomeness {
		Very,
		Somewhat,
		NotReally,
	}
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
		e::strum::IntoStaticStr,
	)]
	pub enum Smartness {
		#[serde(rename = "low")]
		#[strum(serialize = "low")]
		Low,
		#[serde(rename = "mid")]
		#[strum(serialize = "mid")]
		Mid,
		#[serde(rename = "genius")]
		#[strum(serialize = "genius")]
		Genius,
	}
}

fn main() {}
