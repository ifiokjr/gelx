#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cargo.md"))]
//! ## Features
#![doc = document_features::document_features!()]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
#[macro_export]
macro_rules! gelx {
	($module:ident, $query:literal) => {
		$crate::exports::gelx_macros::gelx_raw!($module, query: $query);
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
#[macro_export]
macro_rules! gelx_file {
	($module:ident, $path:literal) => {
		$crate::exports::gelx_macros::gelx_raw!($module, file: $path);
	};
}

pub mod exports {
	#[cfg(feature = "with_bigdecimal")]
	#[cfg_attr(docsrs, doc(cfg(feature = "with_bigdecimal")))]
	pub use bigdecimal;
	pub use bytes;
	#[cfg(feature = "with_chrono")]
	#[cfg_attr(docsrs, doc(cfg(feature = "with_chrono")))]
	pub use chrono;
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
	#[cfg(feature = "builder")]
	#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
	pub use typed_builder;
	pub use uuid;
}
