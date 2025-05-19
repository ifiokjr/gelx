mod gelx_generated;

use gelx::create_client;
use gelx_generated::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let client = create_client().await?;
	let props = select_user::Input::builder().slug("test").build();
	let query = select_user::query(&client, &props).await?;

	println!("{query:?}");
	Ok(())
}
