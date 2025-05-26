#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ifiokjr/gelx/main/setup/assets/logo.png")]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]

use std::env;
use std::path::PathBuf;

pub use gelx_core::GelxCoreError;
pub use gelx_core::GelxCoreResult;
pub use gelx_core::GelxMetadata;
pub use tokio;
use tokio::fs;
use tokio::runtime::Runtime;

/// Enables reading from the configuration in the `Cargo.toml`. This returns the
/// [`GelxMetadata`] struct which can be used to customise the generated code
/// via [`set_metadata_env`].
///
/// ```
/// // build.rs
/// use gelx_build::GelxCoreResult;
/// use gelx_build::gelx_build;
/// use gelx_build::tokio;
///
/// #[tokio::main]
/// async fn main() -> GelxCoreResult<()> {
/// 	gelx_build().await?;
/// 	Ok(())
/// }
/// ```
///
/// To customise the generated code, you can use the [`GelxMetadata`] struct
/// to set the [`GelxMetadata::queries_path`] and [`GelxMetadata::features`]
/// fields.
///
/// ```
/// // build.rs
/// use std::path::PathBuf;
///
/// use gelx_build::GelxCoreResult;
/// use gelx_build::gelx_build;
/// use gelx_build::set_metadata_env;
/// use gelx_build::tokio;
/// use gelx_core::GelxFeatures;
///
/// #[tokio::main]
/// async fn main() -> GelxCoreResult<()> {
/// 	let mut metadata = gelx_build().await?;
/// 	metadata.queries_path = PathBuf::from("queries");
/// 	metadata.features = GelxFeatures::default();
/// 	set_metadata_env(&metadata)?;
///
/// 	Ok(())
/// }
/// ```
pub async fn gelx_build() -> GelxCoreResult<GelxMetadata> {
	let manifest_dir =
		PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
	let cargo_toml_path = manifest_dir.join("Cargo.toml");
	println!("cargo::rerun-if-changed={}", cargo_toml_path.display());
	let cargo_toml_contents = fs::read_to_string(&cargo_toml_path).await?;
	// TODO: support output to OUT_DIR with shared enums rather than generating the
	// enum every time.
	let metadata = GelxMetadata::try_from(cargo_toml_contents).unwrap_or_default();
	set_metadata_env(&metadata)?;

	Ok(metadata)
}

/// Sets the `GELX_METADATA_BASE64` environment variable to the base64 encoded
/// [`GelxMetadata`] struct.
pub fn set_metadata_env(metadata: &GelxMetadata) -> GelxCoreResult<()> {
	let metadata_base64 = metadata.try_to_base64()?;
	println!("cargo::rustc-env=GELX_METADATA_BASE64={metadata_base64}");
	Ok(())
}

/// Enables reading from the configuration in the `Cargo.toml` in a sync
/// environment. See [`gelx_build`] for more information.
///
/// ```
/// // build.rs
/// use gelx_build::gelx_build_sync;
/// use gelx_core::GelxCoreResult;
///
/// fn main() -> GelxCoreResult<()> {
/// 	let metadata = gelx_build_sync()?;
/// 	Ok(())
/// }
/// ```
pub fn gelx_build_sync() -> GelxCoreResult<GelxMetadata> {
	let rt = Runtime::new()?;
	rt.block_on(async { gelx_build().await })
}
