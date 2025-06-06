pub use self::generate::*;
pub use self::globals::*;
pub use self::modules::*;
pub use self::types::*;

mod generate;
mod globals;
mod modules;
mod types;

#[cfg(test)]
mod tests {
	use quote::format_ident;

	use super::*;
	use crate::GelxCoreResult;
	use crate::GelxMetadata;

	#[tokio::test]
	async fn test_generate_enum() -> GelxCoreResult<()> {
		let metadata = GelxMetadata::default();

		generate_module_outputs(&metadata).await?;

		Ok(())
	}

	#[test]
	fn test_module_name() {
		let original = "test::test2::Amazing";
		let module_name = ModuleName::from(original);

		assert_eq!(module_name.original_name(), original);
		assert_eq!(
			module_name.modules_path().unwrap(),
			syn::parse_str("test::test2").unwrap()
		);
		assert_eq!(module_name.name_ident(true), format_ident!("amazing"));
		assert_eq!(module_name.name_ident(false), format_ident!("Amazing"));
	}
}
