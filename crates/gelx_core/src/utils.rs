use std::env;
use std::path::Path;
use std::path::PathBuf;

use gel_protocol::codec::CAL_DATE_DURATION;
use gel_protocol::codec::CAL_LOCAL_DATE;
use gel_protocol::codec::CAL_LOCAL_DATETIME;
use gel_protocol::codec::CAL_LOCAL_TIME;
use gel_protocol::codec::CAL_RELATIVE_DURATION;
use gel_protocol::codec::CFG_MEMORY;
use gel_protocol::codec::PGVECTOR_VECTOR;
use gel_protocol::codec::POSTGIS_GEOGRAPHY;
use gel_protocol::codec::POSTGIS_GEOMETRY;
use gel_protocol::codec::STD_BIGINT;
use gel_protocol::codec::STD_BOOL;
use gel_protocol::codec::STD_BYTES;
use gel_protocol::codec::STD_DATETIME;
use gel_protocol::codec::STD_DECIMAL;
use gel_protocol::codec::STD_DURATION;
use gel_protocol::codec::STD_FLOAT32;
use gel_protocol::codec::STD_FLOAT64;
use gel_protocol::codec::STD_INT16;
use gel_protocol::codec::STD_INT32;
use gel_protocol::codec::STD_INT64;
use gel_protocol::codec::STD_JSON;
use gel_protocol::codec::STD_PG_DATE;
use gel_protocol::codec::STD_PG_JSON;
use gel_protocol::codec::STD_PG_TIMESTAMP;
use gel_protocol::codec::STD_PG_TIMESTAMPTZ;
use gel_protocol::codec::STD_STR;
use gel_protocol::codec::STD_UUID;
use gel_protocol::model::Uuid;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn uuid_to_token_name(uuid: &Uuid, exports_ident: &Ident) -> TokenStream {
	maybe_uuid_to_token_name(uuid, exports_ident)
		.unwrap_or(quote!(#exports_ident::gel_protocol::value::Value))
}

pub(crate) fn maybe_uuid_to_token_name(uuid: &Uuid, exports_ident: &Ident) -> Option<TokenStream> {
	match *uuid {
		STD_UUID => Some(quote!(#exports_ident::uuid::Uuid)),
		STD_STR => Some(quote!(String)),
		STD_BYTES => Some(quote!(#exports_ident::bytes::Bytes)),
		STD_INT16 => Some(quote!(i16)),
		STD_INT32 => Some(quote!(i32)),
		STD_INT64 => Some(quote!(i64)),
		STD_FLOAT32 => Some(quote!(f32)),
		STD_FLOAT64 => Some(quote!(f64)),
		STD_DECIMAL => Some(quote!(#exports_ident::DecimalAlias)),
		STD_BOOL => Some(quote!(bool)),
		STD_DATETIME | STD_PG_TIMESTAMPTZ => Some(quote!(#exports_ident::DateTimeAlias)),
		CAL_LOCAL_DATETIME | STD_PG_TIMESTAMP => Some(quote!(#exports_ident::LocalDatetimeAlias)),
		CAL_LOCAL_DATE | STD_PG_DATE => Some(quote!(#exports_ident::LocalDateAlias)),
		CAL_LOCAL_TIME => Some(quote!(#exports_ident::LocalTimeAlias)),
		STD_DURATION => Some(quote!(#exports_ident::gel_protocol::model::Duration)),
		CAL_RELATIVE_DURATION => {
			Some(quote!(#exports_ident::gel_protocol::model::RelativeDuration))
		}
		CAL_DATE_DURATION => Some(quote!(#exports_ident::gel_protocol::model::DateDuration)),
		STD_JSON | STD_PG_JSON => Some(quote!(#exports_ident::gel_protocol::model::Json)),
		STD_BIGINT => Some(quote!(#exports_ident::BigIntAlias)),
		CFG_MEMORY => Some(quote!(#exports_ident::gel_protocol::model::ConfigMemory)),
		PGVECTOR_VECTOR => Some(quote!(#exports_ident::gel_protocol::model::Vector)),
		POSTGIS_GEOMETRY => Some(quote!(#exports_ident::Geometry)),
		POSTGIS_GEOGRAPHY => Some(quote!(#exports_ident::Geography)),
		_ => None,
	}
}

pub(crate) fn maybe_uuid_to_import(uuid: &Uuid, exports_ident: &Ident) -> Option<TokenStream> {
	match *uuid {
		STD_UUID => Some(quote!(#exports_ident::gel_protocol::codec::STD_UUID)),
		STD_STR => Some(quote!(#exports_ident::gel_protocol::codec::STD_STR)),
		STD_BYTES => Some(quote!(#exports_ident::gel_protocol::codec::STD_BYTES)),
		STD_INT16 => Some(quote!(#exports_ident::gel_protocol::codec::STD_INT16)),
		STD_INT32 => Some(quote!(#exports_ident::gel_protocol::codec::STD_INT32)),
		STD_INT64 => Some(quote!(#exports_ident::gel_protocol::codec::STD_INT64)),
		STD_FLOAT32 => Some(quote!(#exports_ident::gel_protocol::codec::STD_FLOAT32)),
		STD_FLOAT64 => Some(quote!(#exports_ident::gel_protocol::codec::STD_FLOAT64)),
		STD_DECIMAL => Some(quote!(#exports_ident::gel_protocol::codec::STD_DECIMAL)),
		STD_BOOL => Some(quote!(#exports_ident::gel_protocol::codec::STD_BOOL)),
		STD_DATETIME | STD_PG_TIMESTAMPTZ => {
			Some(quote!(#exports_ident::gel_protocol::codec::STD_DATETIME))
		}
		CAL_LOCAL_DATETIME | STD_PG_TIMESTAMP => {
			Some(quote!(#exports_ident::gel_protocol::codec::CAL_LOCAL_DATETIME))
		}
		CAL_LOCAL_DATE | STD_PG_DATE => {
			Some(quote!(#exports_ident::gel_protocol::codec::CAL_LOCAL_DATE))
		}
		CAL_LOCAL_TIME => Some(quote!(#exports_ident::gel_protocol::codec::CAL_LOCAL_TIME)),
		STD_DURATION => Some(quote!(#exports_ident::gel_protocol::codec::STD_DURATION)),
		CAL_RELATIVE_DURATION => {
			Some(quote!(#exports_ident::gel_protocol::codec::CAL_RELATIVE_DURATION ))
		}
		CAL_DATE_DURATION => Some(quote!(#exports_ident::gel_protocol::codec::CAL_DATE_DURATION)),
		STD_JSON | STD_PG_JSON => Some(quote!(#exports_ident::gel_protocol::codec::STD_JSON)),
		STD_BIGINT => Some(quote!(#exports_ident::gel_protocol::codec::STD_BIGINT)),
		CFG_MEMORY => Some(quote!(#exports_ident::gel_protocol::codec::CFG_MEMORY)),
		PGVECTOR_VECTOR => Some(quote!(#exports_ident::gel_protocol::codec::PGVECTOR_VECTOR)),
		POSTGIS_GEOMETRY => Some(quote!(#exports_ident::gel_protocol::codec::POSTGIS_GEOMETRY)),
		POSTGIS_GEOGRAPHY => Some(quote!(#exports_ident::gel_protocol::codec::POSTGIS_GEOGRAPHY)),
		_ => None,
	}
}

/// Taken from <https://github.com/launchbadge/sqlx/blob/f69f370f25f099fd5732a5383ceffc76f724c482/sqlx-macros-core/src/common.rs#L1C1-L37C2>
pub fn resolve_path(path: impl AsRef<Path>, error_span: Span) -> syn::Result<PathBuf> {
	let path = path.as_ref();

	if path.is_absolute() {
		return Err(syn::Error::new(
			error_span,
			"absolute paths will only work on the current machine",
		));
	}

	// requires `proc_macro::SourceFile::path()` to be stable
	// https://github.com/rust-lang/rust/issues/54725
	if path.is_relative()
		&& path
			.parent()
			.is_none_or(|parent| parent.as_os_str().is_empty())
	{
		return Err(syn::Error::new(
			error_span,
			"paths relative to the current file's directory are not currently supported",
		));
	}

	let base_dir = env::var("CARGO_MANIFEST_DIR").map_err(|_| {
		syn::Error::new(
			error_span,
			"CARGO_MANIFEST_DIR is not set; please use Cargo to build",
		)
	})?;
	let base_dir_path = Path::new(&base_dir);

	Ok(base_dir_path.join(path))
}

/// Will format the given source code using `prettyplease`.
pub fn prettify(source: &str) -> syn::Result<String> {
	Ok(prettyplease::unparse(&syn::parse_str(source)?))
}

#[cfg(test)]
mod tests {
	use assert2::check;
	use quote::format_ident;
	use rstest::rstest;
	use syn::Ident;

	use super::*;
	use crate::GelxCoreResult;

	#[test]
	fn can_format_file() -> GelxCoreResult<()> {
		let content = "struct Foo { content: String, allowed: bool, times: u64 }";
		let formatted = prettify(content)?;

		insta::assert_snapshot!(formatted);

		Ok(())
	}

	#[test]
	fn error_when_formatting_invalid_rust() {
		let content = "struct Foo { content: String, allowed: bool, times: u64,,,,, INVALID}";
		let result = prettify(content);

		check!(result.is_err(), "result should be an error");
	}

	#[rstest]
	#[case::std_uuid(STD_UUID, "exports::uuid::Uuid")]
	#[case::std_str(STD_STR, "String")]
	#[case::std_bytes(STD_BYTES, "exports::bytes::Bytes")]
	#[case::std_int16(STD_INT16, "i16")]
	#[case::std_int32(STD_INT32, "i32")]
	#[case::std_int64(STD_INT64, "i64")]
	#[case::std_float32(STD_FLOAT32, "f32")]
	#[case::std_float64(STD_FLOAT64, "f64")]
	#[case::std_decimal(STD_DECIMAL, "exports::DecimalAlias")]
	#[case::std_bool(STD_BOOL, "bool")]
	#[case::std_datetime(STD_DATETIME, "exports::DateTimeAlias")]
	#[case::std_pg_timestamptz(STD_PG_TIMESTAMPTZ, "exports::DateTimeAlias")]
	#[case::cal_local_datetime(CAL_LOCAL_DATETIME, "exports::LocalDatetimeAlias")]
	#[case::std_pg_timestamp(STD_PG_TIMESTAMP, "exports::LocalDatetimeAlias")]
	#[case::cal_local_date(CAL_LOCAL_DATE, "exports::LocalDateAlias")]
	#[case::std_pg_date(STD_PG_DATE, "exports::LocalDateAlias")]
	#[case::cal_local_time(CAL_LOCAL_TIME, "exports::LocalTimeAlias")]
	#[case::std_duration(STD_DURATION, "exports::gel_protocol::model::Duration")]
	#[case::cal_relative_duration(
		CAL_RELATIVE_DURATION,
		"exports::gel_protocol::model::RelativeDuration"
	)]
	#[case::cal_date_duration(CAL_DATE_DURATION, "exports::gel_protocol::model::DateDuration")]
	#[case::std_json(STD_JSON, "exports::gel_protocol::model::Json")]
	#[case::std_pg_json(STD_PG_JSON, "exports::gel_protocol::model::Json")]
	#[case::std_bigint(STD_BIGINT, "exports::BigIntAlias")]
	#[case::cfg_memory(CFG_MEMORY, "exports::gel_protocol::model::ConfigMemory")]
	#[case::pgvector_vector(PGVECTOR_VECTOR, "exports::gel_protocol::model::Vector")]
	#[case::postgis_geometry(POSTGIS_GEOMETRY, "exports::Geometry")]
	#[case::postgis_geography(POSTGIS_GEOGRAPHY, "exports::Geography")]
	fn test_maybe_uuid_to_token_name(#[case] uuid: Uuid, #[case] expected: &str) {
		let exports_ident = format_ident!("exports");
		let result = maybe_uuid_to_token_name(&uuid, &exports_ident);
		assert!(
			result
				.unwrap()
				.to_string()
				.replace(' ', "")
				.contains(expected)
		);
	}

	#[test]
	fn test_maybe_uuid_to_token_name_none() {
		let exports_ident = format_ident!("exports");
		let default_uuid = Uuid::default();
		let result = maybe_uuid_to_token_name(&default_uuid, &exports_ident);
		assert!(result.is_none());
	}

	#[rstest]
	#[case::std_uuid(STD_UUID, "STD_UUID")]
	#[case::std_str(STD_STR, "STD_STR")]
	#[case::std_bytes(STD_BYTES, "STD_BYTES")]
	#[case::std_int16(STD_INT16, "STD_INT16")]
	#[case::std_int32(STD_INT32, "STD_INT32")]
	#[case::std_int64(STD_INT64, "STD_INT64")]
	#[case::std_float32(STD_FLOAT32, "STD_FLOAT32")]
	#[case::std_float64(STD_FLOAT64, "STD_FLOAT64")]
	#[case::std_decimal(STD_DECIMAL, "STD_DECIMAL")]
	#[case::std_bool(STD_BOOL, "STD_BOOL")]
	#[case::std_datetime(STD_DATETIME, "STD_DATETIME")]
	#[case::std_pg_timestamptz(STD_PG_TIMESTAMPTZ, "STD_DATETIME")]
	#[case::cal_local_datetime(CAL_LOCAL_DATETIME, "CAL_LOCAL_DATETIME")]
	#[case::std_pg_timestamp(STD_PG_TIMESTAMP, "CAL_LOCAL_DATETIME")]
	#[case::cal_local_date(CAL_LOCAL_DATE, "CAL_LOCAL_DATE")]
	#[case::std_pg_date(STD_PG_DATE, "CAL_LOCAL_DATE")]
	#[case::cal_local_time(CAL_LOCAL_TIME, "CAL_LOCAL_TIME")]
	#[case::std_duration(STD_DURATION, "STD_DURATION")]
	#[case::cal_relative_duration(CAL_RELATIVE_DURATION, "CAL_RELATIVE_DURATION")]
	#[case::cal_date_duration(CAL_DATE_DURATION, "CAL_DATE_DURATION")]
	#[case::std_json(STD_JSON, "STD_JSON")]
	#[case::std_pg_json(STD_PG_JSON, "STD_JSON")]
	#[case::std_bigint(STD_BIGINT, "STD_BIGINT")]
	#[case::cfg_memory(CFG_MEMORY, "CFG_MEMORY")]
	#[case::pgvector_vector(PGVECTOR_VECTOR, "PGVECTOR_VECTOR")]
	#[case::postgis_geometry(POSTGIS_GEOMETRY, "POSTGIS_GEOMETRY")]
	#[case::postgis_geography(POSTGIS_GEOGRAPHY, "POSTGIS_GEOGRAPHY")]
	fn test_maybe_uuid_to_import(#[case] uuid: Uuid, #[case] expected: &str) {
		let exports_ident = Ident::new("exports", Span::call_site());
		let result = maybe_uuid_to_import(&uuid, &exports_ident);
		assert!(
			result
				.unwrap()
				.to_string()
				.replace(' ', "")
				.contains(expected)
		);
	}

	#[test]
	fn test_maybe_uuid_to_import_none() {
		let exports_ident = Ident::new("exports", Span::call_site());
		let default_uuid = Uuid::default(); // A UUID not in the match arms
		let result = maybe_uuid_to_import(&default_uuid, &exports_ident);
		assert!(result.is_none());
	}
}
