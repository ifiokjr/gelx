#![cfg(all(feature = "query", feature = "serde"))]

use gel_tokio::create_client;
use gelx::gelx;
use gelx_core::Result;

gelx!(insert_user);
gelx!(remove_user);
gelx!(empty_set, r#"select <int64>{}"#);
gelx!(
	simple,
	r#"select {hello := "world", custom := <str>$custom }"#
);

#[tokio::test]
pub async fn simple_query_with_input() -> Result<()> {
	let client = create_client().await?;
	let input = simple::Input {
		custom: String::from("This is a custom field"),
	};
	let output = simple::query(&client, &input).await?;

	insta::assert_ron_snapshot!(output, @r###"
 Output(
   hello: "world",
   custom: "This is a custom field",
 )
 "###);

	Ok(())
}

#[tokio::test]
pub async fn empty_set_query() -> Result<()> {
	let client = create_client().await?;
	let output = empty_set::query(&client).await?;

	insta::assert_ron_snapshot!(output, @"None");

	Ok(())
}

#[tokio::test]
pub async fn run_query() -> Result<()> {
	let client = create_client().await?;

	let insert_props = insert_user::Input::builder()
		.name("Test Query")
		.bio("A biography of immense accomplishment")
		.slug("test_query")
		.build();
	let result = insert_user::query(&client, &insert_props).await?;
	insta::assert_ron_snapshot!(result, {	".id" => "[uuid]"	}, @r###"
 Output(
   id: "[uuid]",
   name: Some("Test Query"),
   bio: Some("A biography of immense accomplishment"),
   slug: "test_query",
 )
 "###);

	// cleanup
	let remove_props = remove_user::Input::builder().id(result.id).build();
	remove_user::query(&client, &remove_props).await?;

	Ok(())
}

#[tokio::test]
pub async fn run_transaction() -> Result<()> {
	let client = create_client().await?;

	let result = client
		.transaction(|mut tx| {
			async move {
				let insert_props = insert_user::Input::builder()
					.name("Test Transaction")
					.bio("another bio of class")
					.slug("test_transaction")
					.build();
				let result = insert_user::transaction(&mut tx, &insert_props).await?;

				// cleanup
				let remove_props = remove_user::Input::builder().id(result.id).build();
				remove_user::transaction(&mut tx, &remove_props).await?;

				Ok(result)
			}
		})
		.await?;

	insta::assert_ron_snapshot!(result, {	".id" => "[uuid]"	}, @r###"
 Output(
   id: "[uuid]",
   name: Some("Test Transaction"),
   bio: Some("another bio of class"),
   slug: "test_transaction",
 )
 "###);

	Ok(())
}
