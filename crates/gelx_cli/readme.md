# `gelx_cli`

<br />

> A command-line interface for `gelx` to generate fully typed Rust code from your Gel schema and query files.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Installation

The `gelx_cli` is distributed as part of the `gelx` workspace. To use it, you can build the workspace and then run the `gelx` binary from the target directory, or install it using `cargo`:

```bash
cargo install gelx_cli
```

Alternatively, you can install it directly from the github repo:

```bash
cargo install --git https://github.com/ifiokjr/gelx.git gelx_cli
```

The above two commands are the typical ways to install `gelx_cli`. However, if you use `gel` in multiple projects with multiple versions of the `gelx` crate, you should install it locally per project using [`cargo-run-bin`](https://crates.io/crates/cargo-run-bin).

First install `cargo-run-bin`:

```bash
cargo install cargo-run-bin
```

Inside a single crate project add the following to the `Cargo.toml` file:

```toml
[package.metadata.bin]
gelx_cli = { version = "0.7.0", bins = ["gelx"] }
```

Inside a workspace add the following to the `Cargo.toml` file:

```toml
[workspace.metadata.bin]
gelx_cli = { version = "0.7.0", bins = ["gelx"] }
```

Then run the following command to access gelx:

```bash
cargo bin gelx
```

Ensure your Gel instance is running and accessible, as the CLI needs to connect to it to introspect the schema and query types.

Run `gelx help` to see the available commands and options.

```bash
Generate fully typed rust code from your gel schema and inline queries.

Usage: gelx [OPTIONS] <COMMAND>

Commands:
  generate  Generates Rust code from the crate in the current directory
  check     Checks if the generated Rust code is up-to-date
  help      Print this message or the help of the given subcommand(s)

Options:
      --cwd <CWD>  Optional working directory to run the command from
  -h, --help       Print help
  -V, --version    Print version
```

## Global Options

- `--cwd <path>`: Specifies a working directory to run the command from. If provided, `gelx` will change to this directory before performing any operations. This is useful if you are invoking `gelx` from a directory different from your project's root.

## Usage

The `gelx` CLI tool generates Rust code from `.edgeql` files located in your project. It reads configuration from your crate\'s `Cargo.toml` file, specifically under the `[package.metadata.gelx]` section.

### Commands

#### `gelx generate`

```bash
Generates Rust code from the crate in the current directory

Usage: gelx generate [OPTIONS]

Options:
      --json       Print the generated code as JSON to stdout instead of writing to a file
      --cwd <CWD>  Optional working directory to run the command from
  -h, --help       Print help
```

This command generates Rust code based on your `.edgeql` query files and the Gel schema.

```bash
gelx generate --cwd path/to/your/crate
```

Or run the command with the `--json` flag to print the generated code to the terminal as JSON:

```bash
gelx generate --cwd path/to/your/crate --json
```

The CLI will:

- Read configuration from `[package.metadata.gelx]` in your `Cargo.toml`.
- Scan the directory specified by `queries` (default: `./queries`) for `.edgeql` files.
- Connect to your Gel instance to get type information for each query.
- Generate corresponding Rust modules.
- If `--json` is used, print the combined code to the terminal as JSON.
- Otherwise, write the combined code to the folder specified by `output_path` (default: `./src/db`).

#### `gelx check`

```bash
Checks if the generated Rust code is up-to-date

Usage: gelx check [OPTIONS]

Options:
      --cwd <CWD>  Optional working directory to run the command from
  -h, --help       Print help
```

This command verifies if the currently generated code is up-to-date with your schema and query files. It\'s useful for CI pipelines to ensure that code generation has been run after any changes.

```bash
gelx check --cwd path/to/your/crate
```

The CLI will:

- Perform the same generation process as `gelx generate` in memory.
- Compare the newly generated code with the content of the existing `output_file`.
- If they match, it will exit successfully (status code 0).
- If they differ, it will print an error message and exit with a non-zero status code, indicating that `gelx generate` needs to be run.

## Configuration

The `gelx` CLI reads its configuration from the `Cargo.toml` file of the crate it is being run in. The configuration should be placed under the `[package.metadata.gelx]` table.

Refer to the main `gelx` crate [readme.md](https://github.com/ifiokjr/gelx/blob/main/readme.md#configuration) for all the configuration options.

## Contributing

This crate is part of the `gelx` workspace. Please refer to the [main project's contributing guide](https://github.com/ifiokjr/gelx/blob/main/CONTRIBUTING.md) for details on how to set up the development environment and contribute.

## License

Unlicense, see the [license file](https://github.com/ifiokjr/gelx/blob/main/LICENSE) in the root of the workspace.

[crate-image]: https://img.shields.io/crates/v/gelx_cli.svg
[crate-link]: https://crates.io/crates/gelx_cli
[docs-image]: https://docs.rs/gelx_cli/badge.svg
[docs-link]: https://docs.rs/gelx_cli/
[ci-status-image]: https://github.com/ifiokjr/gelx/workflows/ci/badge.svg?branch=main
[ci-status-link]: https://github.com/ifiokjr/gelx/actions?query=workflow%3Aci+branch%3Amain
[unlicense-image]: https://img.shields.io/badge/license-Unlicense-blue.svg
[unlicense-link]: https://github.com/ifiokjr/gelx/blob/main/LICENSE
[codecov-image]: https://codecov.io/github/ifiokjr/gelx/graph/badge.svg?token=87K799Q78I
[codecov-link]: https://codecov.io/github/ifiokjr/gelx
