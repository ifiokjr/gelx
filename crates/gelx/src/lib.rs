#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ifiokjr/gelx/main/setup/assets/logo.png")]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]
//! <br />
//! ## Features
#![doc = document_features::document_features!()]

#[cfg(feature = "query")]
pub use gel_tokio::create_client;

/// Generates a query module from a query string.
///
/// ```rust
/// use gelx::gelx;
///
/// gelx!(get_users, "select User {**}");
/// ```
///
/// This macro can be called with one argument if in the root of your crate you
/// host a folder named `queries`.
///
/// ```rust
/// use gelx::gelx;
///
/// gelx!(insert_user);
/// ```
///
/// The above code will find the file `<CRATE_ROOT>/queries/insert_user.edgeql`
/// and run the query from there.
///
/// If you want to customise the module name generated for a specific query
/// path, or just use a custom path for the generated macro in your codebase the
/// following options is also available.
///
/// ```rust
/// use gelx::gelx;
///
/// gelx!(custom_name_for_module, file: "queries/insert_user.edgeql");
/// ```
#[macro_export]
macro_rules! gelx {
	($module:ident, $query:literal) => {
		$crate::exports::gelx_macros::gelx_raw!($module, query: $query);
	};
	($module:ident, file: $path:literal) => {
		$crate::exports::gelx_macros::gelx_raw!($module, file: $path);
	};
	($module: ident) => {
		$crate::exports::gelx_macros::gelx_raw!($module);
	};
}

/// Generates a query module from a query string relative to the root of the
/// crate this is defined in. This is useful for queries that are not placed in
/// the `queries` folder at the root of the crate.
///
/// ```rust
/// use gelx::gelx_file;
///
/// gelx_file!(insert_user, "queries/insert_user.edgeql");
/// ```
///
/// The above code can actually be replaced with the
/// `gelx!(insert_user)` macro since the file is placed in the `queries`
/// folder.
#[deprecated(
	since = "0.4.1",
	note = "use `gelx!(insert_user, file: \"queries/insert_user.edgeql\")` instead"
)]
#[macro_export]
macro_rules! gelx_file {
	($module:ident, $path:literal) => {
		$crate::exports::gelx_macros::gelx_raw!($module, file: $path);
	};
}

pub mod exports {
	pub use bytes;
	use cfg_if::cfg_if;
	#[cfg(feature = "query")]
	#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
	pub use gel_derive;
	pub use gel_errors;
	pub use gel_protocol;
	#[cfg(feature = "query")]
	#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
	pub use gel_tokio;
	pub use gelx_macros;
	#[cfg(any(feature = "with_bigdecimal", feature = "with_bigint"))]
	#[cfg_attr(
		docsrs,
		doc(cfg(any(feature = "with_bigdecimal", feature = "with_bigint")))
	)]
	pub use num_bigint;
	#[cfg(any(feature = "with_bigdecimal", feature = "with_bigint"))]
	#[cfg_attr(
		docsrs,
		doc(cfg(any(feature = "with_bigdecimal", feature = "with_bigint")))
	)]
	pub use num_traits;
	#[cfg(feature = "serde")]
	#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
	pub use serde;
	#[cfg(feature = "strum")]
	#[cfg_attr(docsrs, doc(cfg(feature = "strum")))]
	pub use strum;
	#[cfg(feature = "builder")]
	#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
	pub use typed_builder;
	pub use uuid;

	cfg_if! {
		if #[cfg(feature = "with_bigdecimal")] {
			#[cfg_attr(docsrs, doc(cfg(feature = "with_bigdecimal")))]
			pub use bigdecimal;
			pub type DecimalAlias = bigdecimal::BigDecimal;
		} else {
			pub type DecimalAlias = gel_protocol::model::Decimal;
		}
	}

	cfg_if! {
		if #[cfg(feature = "with_chrono")] {
			#[cfg_attr(docsrs, doc(cfg(feature = "with_chrono")))]
			pub use chrono;
			pub type DateTimeAlias = chrono::DateTime<chrono::Utc>;
			pub type LocalDatetimeAlias = chrono::NaiveDateTime;
			pub type LocalDateAlias = chrono::NaiveDate;
			pub type LocalTimeAlias = chrono::NaiveTime;
		} else {
			pub type DateTimeAlias = gel_protocol::model::Datetime;
			pub type LocalDatetimeAlias = gel_protocol::model::LocalDatetime;
			pub type LocalTimeAlias = gel_protocol::model::LocalTime;
			pub type LocalDateAlias = gel_protocol::model::LocalDate;
		}
	}

	cfg_if! {
		if #[cfg(feature = "with_bigint")] {
			#[cfg_attr(docsrs, doc(cfg(feature = "with_bigint")))]
			pub type BigIntAlias = num_bigint::BigInt;
		} else {
			pub type BigIntAlias = gel_protocol::model::BigInt;
		}
	}
}
