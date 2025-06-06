use gel_tokio::Client;
use gel_tokio::Error;
use gel_tokio::GlobalsDelta;
pub use gel_tokio::create_client;

/// Create a gel client with the provided globals trait.
pub async fn create_client_with_globals(globals: impl GlobalsDelta) -> Result<Client, Error> {
	let client = create_client().await?.with_globals(globals);

	Ok(client)
}
