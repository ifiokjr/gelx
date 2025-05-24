#![doc(html_logo_url = "https://raw.githubusercontent.com/ifiokjr/gelx/main/setup/assets/logo.png")]

use std::collections::HashSet;
use std::path::PathBuf;

use clap::Parser;
use futures::stream::StreamExt;
use gelx_core::GelxCoreResult;
use gelx_core::GelxMetadata;
use gelx_core::ModuleOutputs;
use gelx_core::generate_module_outputs;
use gelx_core::generate_query_token_stream;
use gelx_core::get_descriptor;
use proc_macro2::TokenStream;
use similar::ChangeTag;
use similar::TextDiff;
use vfs::async_vfs::AsyncMemoryFS;
use vfs::async_vfs::AsyncPhysicalFS;
use vfs::async_vfs::AsyncVfsPath;

/// A CLI for generating typed Rust code from Gel queries
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
	#[clap(subcommand)]
	command: Commands,

	/// Optional working directory to run the command from.
	#[clap(long, value_parser = clap::value_parser!(PathBuf), global = true)]
	cwd: Option<PathBuf>,
}

#[derive(Parser, Debug)]
enum Commands {
	/// Generates Rust code from the crate in the current directory.
	Generate {
		/// Print the generated code as JSON to stdout instead of writing to the
		/// directory.
		#[clap(long)]
		json: bool,
	},
	/// Checks if the generated Rust code is up-to-date
	Check,
}

#[tokio::main]
async fn main() -> GelxCoreResult<()> {
	let cli = Cli::parse();

	let current_dir = std::env::current_dir()?;

	// Change current directory if --cwd is provided
	if let Some(path) = &cli.cwd {
		let absolute_path = current_dir.join(path);

		if !absolute_path.is_dir() {
			eprintln!(
				"Error: Invalid --cwd path: {} is not a directory or does not exist.",
				path.display()
			);

			std::process::exit(1);
		}

		std::env::set_current_dir(absolute_path)?;
		eprintln!("Running from directory: {}", path.display());
	}

	// Load metadata from Cargo.toml or gelx.toml in the current (potentially
	// changed) directory
	let current_dir = std::env::current_dir()?;
	let metadata = GelxMetadata::try_new(&current_dir)?;
	let root_path = metadata.root_path.clone().unwrap_or(current_dir);
	let fs = AsyncPhysicalFS::new(&root_path);
	let root_path = AsyncVfsPath::from(fs);

	match cli.command {
		Commands::Generate { json } => handle_generate(&metadata, &root_path, json).await,
		Commands::Check => handle_check(&metadata, &root_path).await,
	}
}

async fn handle_generate(
	metadata: &GelxMetadata,
	root_path: &AsyncVfsPath,
	json: bool,
) -> GelxCoreResult<()> {
	eprintln!("Generating code...");
	let outputs = generate_outputs(metadata, root_path).await?;
	let output_path = root_path.join(metadata.output_path.display().to_string())?;

	if json {
		let json = outputs.to_json_value()?;
		println!("{}", serde_json::to_string_pretty(&json)?);
	} else {
		output_path.create_dir_all().await?;
		outputs.write_to_vfs(&output_path).await?;

		eprintln!(
			"Successfully wrote generated code to {}",
			metadata.output_path.display()
		);
	}

	Ok(())
}

enum Comparison {
	/// The file was added.
	Add(String),
	/// The file was removed.
	Remove(String),
	/// There are changes to the file.
	Change(String, Vec<String>),
}

async fn handle_check(metadata: &GelxMetadata, root_path: &AsyncVfsPath) -> GelxCoreResult<()> {
	eprintln!("Checking code...");
	let generated_code = generate_outputs(metadata, root_path).await?;
	let memfs = AsyncMemoryFS::new();
	let existing_output_path = root_path.join(metadata.output_path.display().to_string())?;
	let generated_output_path = AsyncVfsPath::from(memfs);
	generated_code.write_to_vfs(&generated_output_path).await?;

	// compare two vfs paths.

	// if !root_path.join("")

	if !existing_output_path.exists().await? {
		eprintln!(
			"Error: Output file {} does not exist. Run `gelx generate` first.",
			metadata.output_path.display()
		);
		std::process::exit(1);
	}

	let mut shared = HashSet::new();
	let mut comparison = Vec::new();

	while let Some(entry) = generated_output_path.walk_dir().await?.next().await {
		let entry = entry?;

		let Some(relative_path) = entry.as_str().strip_prefix(generated_output_path.as_str())
		else {
			continue;
		};

		let existing_output_path = existing_output_path.join(relative_path)?;

		if !existing_output_path.exists().await? {
			comparison.push(Comparison::Add(relative_path.to_string()));
			continue;
		}

		let generated_content = entry.read_to_string().await?;
		let existing_content = existing_output_path.read_to_string().await?;
		shared.insert(relative_path.to_string());

		if generated_content != existing_content {
			let diff = TextDiff::from_lines(&existing_content, &generated_content);
			let mut changes = Vec::new();

			for change in diff.iter_all_changes() {
				let sign = match change.tag() {
					ChangeTag::Delete => "-",
					ChangeTag::Insert => "+",
					ChangeTag::Equal => " ",
				};
				changes.push(format!("{sign}{change}"));
			}

			comparison.push(Comparison::Change(relative_path.to_string(), changes));
		}
	}

	while let Some(entry) = existing_output_path.walk_dir().await?.next().await {
		let entry = entry?;

		let Some(relative_path) = entry.as_str().strip_prefix(existing_output_path.as_str()) else {
			continue;
		};

		if !shared.contains(relative_path) {
			comparison.push(Comparison::Remove(relative_path.to_string()));
		}
	}

	if comparison.is_empty() {
		eprintln!("Generated code is up-to-date.");
		std::process::exit(0);
	}

	for change in comparison {
		match change {
			Comparison::Add(path) => {
				eprintln!("Added: {}", existing_output_path.join(&path)?.as_str());
			}
			Comparison::Remove(path) => {
				eprintln!("Removed: {}", existing_output_path.join(&path)?.as_str());
			}
			Comparison::Change(path, diffs) => {
				eprintln!("Changed: {}", existing_output_path.join(&path)?.as_str());

				for diff in diffs {
					eprintln!("{diff}");
				}
			}
		}
	}

	std::process::exit(1);
}

async fn generate_outputs(
	metadata: &GelxMetadata,
	root_path: &AsyncVfsPath,
) -> GelxCoreResult<ModuleOutputs> {
	let mut query_tokens = TokenStream::new();
	let queries_path = root_path.join(metadata.queries_path.display().to_string())?;

	if queries_path.is_dir().await? {
		let mut paths = queries_path.read_dir().await?.collect::<Vec<_>>().await;
		paths.sort_by_key(|path| path.as_str().to_string());

		for path in paths {
			if !path.is_file().await? || path.extension().is_none_or(|ext| ext != "edgeql") {
				continue;
			}

			let query_content = path.read_to_string().await?;
			let filename = path.filename();
			let module_name = filename.to_string();

			eprintln!("Processing query: {}", path.as_str());
			let descriptor = get_descriptor(&query_content, metadata).await?;
			let token_stream = generate_query_token_stream(
				&descriptor,
				&module_name,
				&query_content,
				metadata,
				false,
			)?;

			query_tokens.extend(token_stream);
		}
	}

	let mut outputs = generate_module_outputs(metadata).await?;
	outputs.append_to_root(&query_tokens);

	Ok(outputs)
}

#[cfg(test)]
mod tests {
	use std::process::Command;
	use std::process::Stdio;

	use super::*;

	const CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");

	fn cli() -> Command {
		// TODO: fix this breaks on CI as it needs the build to be run first I think
		// Command::new(insta_cmd::get_cargo_bin("gelx"))
		Command::new("gelx")
	}

	#[test]
	fn generate_stdout() {
		let path = PathBuf::from(CRATE_DIR).join("../../examples/gelx_example");
		insta_cmd::assert_cmd_snapshot!(
			cli()
				.arg("generate")
				.arg("--json")
				.current_dir(path)
				.stderr(Stdio::null())
		);
	}
}
