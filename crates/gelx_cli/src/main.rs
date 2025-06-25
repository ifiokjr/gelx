#![doc(html_logo_url = "https://raw.githubusercontent.com/ifiokjr/gelx/main/setup/assets/logo.png")]

use gelx_cli::Cli;
use gelx_cli::prelude::*;
use gelx_core::GelxCoreResult;

#[tokio::main]
async fn main() -> GelxCoreResult<()> {
	let cli = Cli::parse();

	match cli.run().await {
		Ok(()) => std::process::exit(0),
		Err(err) => {
			eprintln!("{err}");
			std::process::exit(1)
		}
	}
}
