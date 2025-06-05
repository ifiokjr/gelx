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
use gel_protocol::codec::POSTGIS_BOX_2D;
use gel_protocol::codec::POSTGIS_BOX_3D;
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
use gel_protocol::codec::STD_PG_INTERVAL;
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
	match *uuid {
		STD_UUID => quote!(#exports_ident::uuid::Uuid),
		STD_STR => quote!(String),
		STD_BYTES => quote!(#exports_ident::bytes::Bytes),
		STD_INT16 => quote!(i16),
		STD_INT32 => quote!(i32),
		STD_INT64 => quote!(i64),
		STD_FLOAT32 => quote!(f32),
		STD_FLOAT64 => quote!(f64),
		STD_DECIMAL => quote!(#exports_ident::DecimalAlias),
		STD_BOOL => quote!(bool),
		STD_DATETIME | STD_PG_TIMESTAMPTZ => quote!(#exports_ident::DateTimeAlias),
		CAL_LOCAL_DATETIME | STD_PG_TIMESTAMP => quote!(#exports_ident::LocalDatetimeAlias),
		CAL_LOCAL_DATE | STD_PG_DATE => quote!(#exports_ident::LocalDateAlias),
		CAL_LOCAL_TIME => quote!(#exports_ident::LocalTimeAlias),
		STD_DURATION => quote!(#exports_ident::gel_protocol::model::Duration),
		CAL_RELATIVE_DURATION => quote!(#exports_ident::gel_protocol::model::RelativeDuration),
		CAL_DATE_DURATION => quote!(#exports_ident::gel_protocol::model::DateDuration),
		STD_JSON | STD_PG_JSON => quote!(#exports_ident::gel_protocol::model::Json),
		STD_BIGINT => quote!(#exports_ident::BigIntAlias),
		CFG_MEMORY => quote!(#exports_ident::gel_protocol::model::ConfigMemory),
		PGVECTOR_VECTOR => quote!(#exports_ident::gel_protocol::model::Vector),
		STD_PG_INTERVAL => todo!("STD_PG_INTERVAL not yet implemented"),
		POSTGIS_GEOMETRY => quote!(#exports_ident::Geometry),
		POSTGIS_GEOGRAPHY => quote!(#exports_ident::Geography),
		POSTGIS_BOX_2D => todo!("POSTGIS_BOX_2D not yet implemented"),
		POSTGIS_BOX_3D => todo!("POSTGIS_BOX_3D not yet implemented"),
		_ => quote!(()),
	}
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
}
