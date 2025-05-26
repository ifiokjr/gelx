mod db;

use db::*;
use gelx::create_client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let client = create_client().await?;
	let props = select_user::Input::builder().slug("test").build();
	let query = select_user::query(&client, &props).await?;
	println!("{query:?}");

	let props = select_accounts::Input::builder()
		.provider(AccountProvider::Github)
		.build();
	let query = select_accounts::query(&client, &props).await?;
	println!("{query:?}");

	Ok(())
}
