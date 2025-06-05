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

/// A wrapper for the[`geo::Geometry`] enum with support for WKB encoding and
/// interop with the `ext::postgis::geometry` type.
///
/// This type can be used as a queryable type in the `ext::postgis::geometry`
/// type.
///
/// TODO SRID support or an alternative would be creating a way to automatically
/// infer the SRID from the geometry: <https://raw.githubusercontent.com/rustprooflabs/srid-bbox/refs/heads/main/srid_bbox.sql>
///
/// ```
/// use gelx::Geometry;
/// use gelx::geo::point;
///
/// let geometry = Geometry(point!(x: 1.0, y: 1.0).into());
/// ```
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

/// Another wrapper struct for the [`geo::Geometry`] enum with support for WKB
/// encoding and interop with the `ext::postgis::geography` type.
///
/// This type can be used as a queryable type in the `ext::postgis::geography`
/// type.
///
/// ```
/// use gelx::Geography;
/// use gelx::geo::point;
///
/// let geography = Geography(point!(x: 1.0, y: 1.0).into());
/// ```
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

pub fn check_scalar(
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
