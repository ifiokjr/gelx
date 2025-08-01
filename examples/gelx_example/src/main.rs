#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	use gelx::exports::uuid::Uuid;
	use gelx_example::*;

	// Create a client with the default globals.
	let client = Globals::builder()
		.current_user_id(Uuid::max())
		.alternative("test")
		.build()
		.into_client()
		.await?;
	let props = select_user::Input::builder().slug("test").build();
	let query = select_user::query(&client, &props).await?;
	println!("{query:?}");

	let props = select_accounts::Input::builder()
		.provider(AccountProvider::Github)
		.build();
	let query = select_accounts::query(&client, &props).await?;
	println!("{query:?}");

	let props = insert_position::Input::builder().position(1).build();
	let query = insert_position::query(&client, &props).await?;
	println!("{:?}", query.position);

	let props = select_test_user::Input::builder()
		.username("custom")
		.build();
	let query = select_test_user::query(&client, &props).await?;
	println!("{query:?}");

	Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
