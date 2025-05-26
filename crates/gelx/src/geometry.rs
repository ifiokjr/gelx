use std::ops::Deref;

use bytes::Bytes;
use gel_protocol::codec::POSTGIS_GEOGRAPHY;
use gel_protocol::codec::POSTGIS_GEOMETRY;
use gel_protocol::descriptors::Descriptor;
use gel_protocol::descriptors::TypePos;
use gel_protocol::errors;
use gel_protocol::errors::DecodeError;
use gel_protocol::queryable::Decoder;
use gel_protocol::queryable::DescriptorContext;
use gel_protocol::queryable::DescriptorMismatch;
use gel_protocol::queryable::Queryable;
use gel_protocol::value::Value;
use geo_traits::to_geo::ToGeoGeometry;
use uuid::Uuid;
use wkb::Endianness;
use wkb::reader::read_wkb;
use wkb::writer::WriteOptions;
use wkb::writer::write_geometry;

#[derive(
	Debug, Clone, derive_more::Deref, derive_more::DerefMut, derive_more::From, derive_more::Into,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Geometry(pub geo::Geometry);

impl Queryable for Geometry {
	type Args = ();

	fn decode(_: &Decoder, (): &Self::Args, buf: &[u8]) -> Result<Self, DecodeError> {
		let geometry: geo::Geometry = match read_wkb(buf) {
			Ok(value) => {
				let Some(value) = value.try_to_geometry() else {
					return errors::MissingRequiredElement.fail();
				};
				value
			}
			Err(_) => return errors::MissingRequiredElement.fail(),
		};

		Ok(Self(geometry))
	}

	fn check_descriptor(
		ctx: &DescriptorContext,
		type_pos: TypePos,
	) -> Result<Self::Args, DescriptorMismatch> {
		check_scalar(ctx, type_pos, POSTGIS_GEOMETRY, "ext::postgis::geometry")?;
		Ok(())
	}
}

impl From<Bytes> for Geometry {
	fn from(value: Bytes) -> Self {
		(&value).into()
	}
}

impl From<&Bytes> for Geometry {
	fn from(value: &Bytes) -> Self {
		value.deref().into()
	}
}

impl From<&[u8]> for Geometry {
	fn from(value: &[u8]) -> Self {
		Self(read_wkb(value).unwrap().to_geometry())
	}
}

impl From<Geometry> for Value {
	fn from(value: Geometry) -> Self {
		let mut buf = vec![];

		write_geometry(
			&mut buf,
			&value.0,
			&WriteOptions {
				endianness: Endianness::BigEndian,
			},
		)
		.unwrap();

		Value::PostGisGeometry(Bytes::from(buf))
	}
}

#[derive(
	Debug, Clone, derive_more::Deref, derive_more::DerefMut, derive_more::From, derive_more::Into,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Geography(pub geo::Geometry);

impl Queryable for Geography {
	type Args = ();

	fn decode(_: &Decoder, (): &Self::Args, buf: &[u8]) -> Result<Self, DecodeError> {
		let geometry: geo::Geometry = match read_wkb(buf) {
			Ok(value) => {
				let Some(value) = value.try_to_geometry() else {
					return errors::MissingRequiredElement.fail();
				};
				value
			}
			Err(_) => return errors::MissingRequiredElement.fail(),
		};

		Ok(Self(geometry))
	}

	fn check_descriptor(
		ctx: &DescriptorContext,
		type_pos: TypePos,
	) -> Result<Self::Args, DescriptorMismatch> {
		check_scalar(ctx, type_pos, POSTGIS_GEOGRAPHY, "ext::postgis::geography")?;
		Ok(())
	}
}

impl From<Bytes> for Geography {
	fn from(value: Bytes) -> Self {
		(&value).into()
	}
}

impl From<&Bytes> for Geography {
	fn from(value: &Bytes) -> Self {
		value.deref().into()
	}
}

impl From<&[u8]> for Geography {
	fn from(value: &[u8]) -> Self {
		Self(read_wkb(value).unwrap().to_geometry())
	}
}

impl From<Geography> for Value {
	fn from(value: Geography) -> Self {
		let mut buf = vec![];

		write_geometry(
			&mut buf,
			&value.0,
			&WriteOptions {
				endianness: Endianness::BigEndian,
			},
		)
		.unwrap();

		Value::PostGisGeography(Bytes::from(buf))
	}
}

fn check_scalar(
	ctx: &DescriptorContext,
	type_pos: TypePos,
	type_id: Uuid,
	name: &str,
) -> Result<(), DescriptorMismatch> {
	let desc = ctx.get(type_pos)?;

	match desc {
		Descriptor::Scalar(scalar) if scalar.base_type_pos.is_some() => {
			return check_scalar(ctx, scalar.base_type_pos.unwrap(), type_id, name);
		}
		Descriptor::Scalar(scalar) if *scalar.id == type_id => {
			return Ok(());
		}
		Descriptor::BaseScalar(base) if *base.id == type_id => {
			return Ok(());
		}
		_ => {}
	}

	Err(ctx.wrong_type(desc, name))
}
