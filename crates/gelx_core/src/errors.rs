use gel_protocol::errors::DecodeError;
use gel_tokio::dsn::error::ParseError;
use proc_macro2::Span;
use thiserror::Error as ThisError;

/// The error type for the `gelx_core` crate.
///
/// Note: This error type is not exhaustive since it will be extended in the
/// future with several automated from implementations.
#[non_exhaustive]
#[derive(Debug, ThisError)]
pub enum GelxCoreError {
	#[error("{0}")]
	Syn(#[from] syn::Error),
	#[error("{0}")]
	Gel(#[from] gel_errors::Error),
	#[error("{0}")]
	GelDsn(#[from] ParseError),
	#[error("{0}")]
	Decode(#[from] DecodeError),
	#[error("{0}")]
	Io(#[from] std::io::Error),
	#[error("{0}")]
	Toml(#[from] toml::de::Error),
	#[error("{0}")]
	TomlDisplay(#[from] toml::ser::Error),
	#[error("{0}")]
	TomlEdit(#[from] toml_edit::TomlError),
	#[error("{0}")]
	SerdeJson(#[from] serde_json::Error),
	#[error("{0}")]
	ProcMacro2(#[from] proc_macro2::LexError),
	#[error("{0}")]
	Base64(#[from] base64::DecodeError),
	#[error("{0}")]
	Utf8(#[from] std::string::FromUtf8Error),
	#[error("{0}")]
	Custom(String),
}

macro_rules! gelx_error {
	($($args:expr),*) => {
		GelxCoreError::Custom(format!($($args),*))
	};
}

pub(crate) use gelx_error;

pub type GelxCoreResult<T> = Result<T, GelxCoreError>;

impl From<GelxCoreError> for syn::Error {
	fn from(error: GelxCoreError) -> Self {
		match error {
			GelxCoreError::Syn(error) => error,
			_ => syn::Error::new(Span::call_site(), error.to_string()),
		}
	}
}
