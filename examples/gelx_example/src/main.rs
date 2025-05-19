mod gelx_generated;

use gelx::create_client;
use gelx_generated::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let client = create_client().await?;
	let query = select_user::query(
		&client,
		&select_user::Input {
			slug: "test".to_string(),
		},
	)
	.await?;

	println!("{query:?}");
	Ok(())
}
