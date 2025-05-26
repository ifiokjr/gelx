use gelx_build::gelx_build;
use gelx_build::gelx_build_sync;
use gelx_core::GelxCoreResult;

#[tokio::test]
async fn build() -> GelxCoreResult<()> {
	insta::assert_snapshot!(gelx_build().await?);

	Ok(())
}

#[test]
fn build_sync() -> GelxCoreResult<()> {
	insta::assert_snapshot!(gelx_build_sync()?);

	Ok(())
}
