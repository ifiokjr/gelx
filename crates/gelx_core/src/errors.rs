use gel_protocol::errors::DecodeError;
use proc_macro2::Span;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
	#[error("{0}")]
	Syn(#[from] syn::Error),
	#[error("{0}")]
	Edgedb(#[from] gel_errors::Error),
	#[error("{0}")]
	Decode(#[from] DecodeError),
	#[error("{0}")]
	Io(#[from] std::io::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<Error> for syn::Error {
	fn from(error: Error) -> Self {
		match error {
			Error::Syn(error) => error,
			_ => syn::Error::new(Span::call_site(), error.to_string()),
		}
	}
}
