use gel_protocol::errors::DecodeError;
use gel_tokio::dsn::error::ParseError;
use proc_macro2::Span;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum GelxError {
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
	TomlEdit(#[from] toml_edit::TomlError),
	#[error("{0}")]
	Custom(String),
}

macro_rules! gelx_error {
	($($args:expr),*) => {
		GelxError::Custom(format!($($args),*))
	};
}

pub(crate) use gelx_error;

pub type GelxResult<T> = Result<T, GelxError>;

impl From<GelxError> for syn::Error {
	fn from(error: GelxError) -> Self {
		match error {
			GelxError::Syn(error) => error,
			_ => syn::Error::new(Span::call_site(), error.to_string()),
		}
	}
}
