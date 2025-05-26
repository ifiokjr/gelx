#![cfg(all(feature = "query", feature = "serde"))]

use gel_tokio::create_client;
use gelx::Geography;
use gelx::Geometry;
use gelx::gelx;
use gelx::geo::point;
use gelx::geo::polygon;
use gelx_core::GelxCoreResult;

gelx!(select_simple_location);
gelx!(insert_location);
gelx!(insert_user);
gelx!(custom_name_for_module, file: "queries/insert_user.edgeql");
gelx!(remove_user);
gelx!(empty_set, r#"select <int64>{}"#);
gelx!(
	simple,
	r#"select {hello := "world", custom := <str>$custom }"#
);

#[tokio::test]
pub async fn select_simple_location_query() -> GelxCoreResult<()> {
	let client = (create_client()).await?;
	let output = (select_simple_location::query(&client)).await?;
	insta::assert_ron_snapshot!(output, @r"
	Geometry(Point(Point(Coord(
	  x: 1.0,
	  y: 1.0,
	))))
	");

	Ok(())
}

#[tokio::test]
pub async fn insert_location_query() -> GelxCoreResult<()> {
	let client = (create_client()).await?;
	let point = point!(x: 1.0, y: 1.0);
	let polygon = polygon![
		(x: -111., y: 45.),
		(x: -111., y: 41.),
		(x: -104., y: 41.),
		(x: -104., y: 45.),
	];
	let output = (insert_location::query(
		&client,
		&insert_location::Input {
			point: Geometry(point.into()),
			area: Geography(polygon.into()),
		},
	))
	.await?;

	insta::assert_ron_snapshot!(output, @r"
	Output(
	  point: Geometry(Point(Point(Coord(
	    x: 1.0,
	    y: 1.0,
	  )))),
	  area: Geography(Polygon(Polygon(
	    exterior: LineString([
	      Coord(
	        x: -111.0,
	        y: 45.0,
	      ),
	      Coord(
	        x: -111.0,
	        y: 41.0,
	      ),
	      Coord(
	        x: -104.0,
	        y: 41.0,
	      ),
	      Coord(
	        x: -104.0,
	        y: 45.0,
	      ),
	      Coord(
	        x: -111.0,
	        y: 45.0,
	      ),
	    ]),
	    interiors: [],
	  ))),
	)
	");

	Ok(())
}

#[tokio::test]
pub async fn simple_query_with_input() -> GelxCoreResult<()> {
	let client = (create_client()).await?;
	let input = simple::Input {
		custom: String::from("This is a custom field"),
	};
	let output = (simple::query(&client, &input)).await?;

	insta::assert_ron_snapshot!(output, @r###"
 Output(
   hello: "world",
   custom: "This is a custom field",
 )
 "###);

	Ok(())
}

#[tokio::test]
pub async fn empty_set_query() -> GelxCoreResult<()> {
	let client = (create_client()).await?;
	let output = (empty_set::query(&client)).await?;

	insta::assert_ron_snapshot!(output, @"None");

	Ok(())
}

#[tokio::test]
pub async fn run_query() -> GelxCoreResult<()> {
	let client = (create_client()).await?;

	let insert_props = insert_user::Input::builder()
		.name("Test Query")
		.bio("A biography of immense accomplishment")
		.slug("test_query")
		.build();
	let result = (insert_user::query(&client, &insert_props)).await?;
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
	let _ = (remove_user::query(&client, &remove_props)).await?;

	Ok(())
}
#[tokio::test]
pub async fn run_custom_name_query() -> GelxCoreResult<()> {
	let client = (create_client()).await?;

	let insert_props = custom_name_for_module::Input::builder()
		.name("Test Query")
		.bio("A biography of immense accomplishment")
		.slug("custom_query")
		.build();
	let result = (custom_name_for_module::query(&client, &insert_props)).await?;
	insta::assert_ron_snapshot!(result, {	".id" => "[uuid]"	}, @r###"
 Output(
   id: "[uuid]",
   name: Some("Test Query"),
   bio: Some("A biography of immense accomplishment"),
   slug: "custom_query",
 )
 "###);

	// cleanup
	let remove_props = remove_user::Input::builder().id(result.id).build();
	let _ = (remove_user::query(&client, &remove_props)).await?;

	Ok(())
}

#[tokio::test]
pub async fn run_transaction() -> GelxCoreResult<()> {
	let client = (create_client()).await?;

	let result = (client.transaction(|mut tx| {
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
	}))
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
