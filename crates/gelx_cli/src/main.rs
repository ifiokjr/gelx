#![doc(html_logo_url = "https://raw.githubusercontent.com/ifiokjr/gelx/main/setup/assets/logo.png")]

use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use gelx_core::GelxCoreResult;
use gelx_core::GelxMetadata;
use gelx_core::ModuleOutputs;
use gelx_core::generate_module_outputs;
use gelx_core::generate_query_token_stream;
use gelx_core::get_descriptor;
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use similar::ChangeTag;
use similar::TextDiff;
use tokio::fs;

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

	match cli.command {
		Commands::Generate { json } => handle_generate(&metadata, &root_path, json).await,
		Commands::Check => handle_check(&metadata, &root_path).await,
	}
}

async fn handle_generate(
	metadata: &GelxMetadata,
	root_path: impl AsRef<Path>,
	json: bool,
) -> GelxCoreResult<()> {
	eprintln!("Generating code...");
	let root_path = root_path.as_ref();
	let outputs = generate_outputs(metadata, root_path).await?;
	let output_path = root_path.join(&metadata.output_path);

	if json {
		let json = outputs.to_map()?;
		println!("{}", serde_json::to_string_pretty(&json)?);
	} else {
		let _ = fs::remove_dir_all(&output_path).await; // clean up the output directory
		fs::create_dir_all(&output_path).await?;
		eprintln!("Writing generated code to {}", output_path.display());
		outputs.write_to_fs(&output_path).await?;

		eprintln!(
			"Successfully wrote generated code to {}",
			metadata.output_path.display()
		);
	}

	Ok(())
}

enum Comparison {
	/// The file was added.
	Add(PathBuf),
	/// The file was removed.
	Remove(PathBuf),
	/// There are changes to the file.
	Change(PathBuf, Vec<String>),
}

async fn handle_check(metadata: &GelxMetadata, root_path: impl AsRef<Path>) -> GelxCoreResult<()> {
	eprintln!("Checking code...");
	let root_path = root_path.as_ref();
	let output_path = root_path.join(&metadata.output_path);

	if !output_path.exists() {
		eprintln!(
			"Error: Output file {} does not exist. Run `gelx generate` first.",
			metadata.output_path.display()
		);
		std::process::exit(1);
	}

	let generated_map = generate_outputs(metadata, root_path).await?.to_map()?;
	println!("generated_map keys: {:?}", generated_map.keys());
	let existing_map = ModuleOutputs::try_new(&output_path, &output_path)
		.await?
		.to_map()?;
	println!("existing_map keys: {:?}", existing_map.keys());
	let mut comparison = Vec::new();

	for (path, content) in &generated_map {
		let Some(existing_content) = existing_map.get(path) else {
			comparison.push(Comparison::Add(path.clone()));
			continue;
		};

		if existing_content != content {
			let diff = TextDiff::from_lines(existing_content, content);
			let mut changes = Vec::new();

			for change in diff.iter_all_changes() {
				let sign = match change.tag() {
					ChangeTag::Delete => "-",
					ChangeTag::Insert => "+",
					ChangeTag::Equal => " ",
				};
				changes.push(format!("{sign}{change}"));
			}

			comparison.push(Comparison::Change(path.clone(), changes));
		}
	}

	for path in existing_map.keys() {
		if !generated_map.contains_key(path) {
			comparison.push(Comparison::Remove(path.clone()));
		}
	}

	if comparison.is_empty() {
		eprintln!("Generated code is up-to-date.");
		std::process::exit(0);
	}

	for change in comparison {
		match change {
			Comparison::Add(path) => {
				eprintln!("Added: {}", root_path.join(&path).display());
			}
			Comparison::Remove(path) => {
				eprintln!("Removed: {}", root_path.join(&path).display());
			}
			Comparison::Change(path, diffs) => {
				eprintln!("Changed: {}", root_path.join(&path).display());

				for diff in diffs {
					eprintln!("{diff}");
				}
			}
		}
	}

	// std::process::exit(1);
	Ok(())
}

async fn generate_outputs(
	metadata: &GelxMetadata,
	root_path: impl AsRef<Path>,
) -> GelxCoreResult<ModuleOutputs> {
	let root_path = root_path.as_ref();
	let mut query_tokens = TokenStream::new();
	let queries_path = root_path.join(&metadata.queries_path);

	if queries_path.is_dir() {
		// not async to make sorting easier
		let mut entries = queries_path.read_dir()?.collect::<Result<Vec<_>, _>>()?;
		entries.sort_by_key(std::fs::DirEntry::path);

		for entry in entries {
			let path = entry.path();

			if !path.is_file() || path.extension().is_none_or(|ext| ext != "edgeql") {
				continue;
			}

			let query_content = fs::read_to_string(&path).await?;
			let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
			let module_name = file_stem.to_snake_case();

			eprintln!("Processing query: {}", path.display());
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
