<p align="center">
  <a href="#">
    <img width="300" src="./setup/assets/logo.svg"  />
  </a>
</p>

<br />

> Generate fully typed rust code from your gel schema and inline queries with `gelx`.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Installation

To install the `gelx` crate you can use the following command.

```bash
cargo add gelx
```

Or directly add the following to your `Cargo.toml` file.

```toml
gelx = "0.3"
```

Follow the [Quickstart Guide](https://docs.gel.com/get-started/quickstart) to make sure your gel instance is running. The macro relies on the running `gel` instance to parse the output of the provided query string.

## Usage

When working with `gel` you often need to write queries and also provide the types for both the input and output. Your code is only checked at runtime which increases the risk of bugs and errors.

Fortunately, `gel` has a query language that is typed and can be converted into types and queried for correctness at compile time.

### Inline Queries

```rust
use gel_errors::Error;
use gel_tokio::create_client;
use gelx::gelx;

// Creates a module called `simple` with a function called `query` and structs
// for the `Input` and `Output`.
gelx!(
	simple,
	"select { hello := \"world\", custom := <str>$custom }"
);

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = create_client().await?;
	let input = simple::Input {
		custom: String::from("custom"),
	};

	// For queries the following code can be used.
	let output = simple::query(&client, &input).await?;

	Ok(())
}
```

The macro above generates the following code:

```rust
pub mod simple {
	use ::gelx::exports as e;
	/// Execute the desired query.
	#[cfg(feature = "query")]
	pub async fn query(
		client: &e::gel_tokio::Client,
		props: &Input,
	) -> core::result::Result<Output, e::gel_errors::Error> {
		client.query_required_single(QUERY, props).await
	}
	/// Compose the query as part of a larger transaction.
	#[cfg(feature = "query")]
	pub async fn transaction(
		conn: &mut e::gel_tokio::Transaction,
		props: &Input,
	) -> core::result::Result<Output, e::gel_errors::Error> {
		conn.query_required_single(QUERY, props).await
	}
	#[derive(Clone, Debug)]
	#[cfg_attr(feature = "builder", derive(e::typed_builder::TypedBuilder))]
	#[cfg_attr(feature = "query", derive(e::gel_derive::Queryable))]
	#[cfg_attr(feature = "serde", derive(e::serde::Serialize, e::serde::Deserialize))]
	pub struct Input {
		#[cfg_attr(feature = "builder", builder(setter(into)))]
		pub custom: String,
	}
	impl e::gel_protocol::query_arg::QueryArgs for Input {
		fn encode(
			&self,
			encoder: &mut e::gel_protocol::query_arg::Encoder,
		) -> core::result::Result<(), e::gel_errors::Error> {
			let map = e::gel_protocol::named_args! {
				"custom" => self.custom.clone(),
			};
			map.encode(encoder)
		}
	}
	#[derive(Clone, Debug)]
	#[cfg_attr(feature = "query", derive(e::gel_derive::Queryable))]
	#[cfg_attr(feature = "serde", derive(e::serde::Serialize, e::serde::Deserialize))]
	pub struct Output {
		pub hello: String,
		pub custom: String,
	}
	/// The original query string provided to the macro. Can be reused in your
	/// codebase.
	pub const QUERY: &str = "select { hello := \"world\", custom := <str>$custom }";
}
```

### Query Files

Define a query file in the `queries` directory of your crate called `select_user.edgeql`.

```edgeql
# queries/select_user.edgeql

select User {
  name,
  bio,
  slug,
} filter .slug = <str>$slug;
```

Then use the `gelx` macro to import the query.

```rust
use gel_errors::Error;
use gel_tokio::create_client;
use gelx::gelx;

// Creates a module called `select_user` with public functions `transaction` and
// `query` as well as structs for the `Input` and `Output`.
gelx_file!(select_user);

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = create_client().await?;

	// Generated code can be run inside a transaction.
	let result = client
		.transaction(|mut txn| {
			async move {
				let input = select_user::Input {
					slug: String::from("test"),
				};
				let output = select_user::transaction(&mut txn, &input).await?;
				Ok(output)
			}
		})
		.await?;

	Ok(())
}
```

## Configuration

The following configuration options are supported.

```toml
[package.metadata.gelx]
## The location of the queries relative to the root of the crate.
queries = "./queries"

## The features to enable and their aliases. By default all features are enabled.
## To disable a feature set it to false. The available features are:
## - query
## - serde
## - strum
## - builder
features = { query = "ssr", strum = "ssr", builder = "ssr" }

## The location of the generated code when using the `gelx` cli.
output_file = "./src/gelx_generated.rs"

## The name of the arguments input struct. Will be transformed to PascalCase.
input_struct_name = "Input"

## The name of the exported output struct for generated queries. Will be transformed to PascalCase.
output_struct_name = "Output"

## The name of the query function exported.
query_function_name = "query"

## The name of the transaction function exported.
transaction_function_name = "transaction"

## The relative path to the `gel` config file. This is optional and if not provided the `gel`
## config will be read from the environment variables.
# gel_config_path = "./gel.toml"

## The name of the `gel` instance to use. This is optional and if not provided the environment
## variable `$GEL_INSTANCE` will be used.
# gel_instance = "$GEL_INSTANCE"

## The name of the `gel` branch to use. This is optional and if not provided the environment
## variable `$GEL_BRANCH` will be used.
# gel_branch = "$GEL_BRANCH"
```

## CLI

The `gelx_cli` crate exposes a binary called `gelx` which can be used to generate the typed code into rust files rather than inline queries.

It should be run from the crate directory and will read from the configuration specified in the previous section.

```bash
cd path/to/crate
gelx generate
```

Sometimes you will need to check that the generated code matches the current database schema and queries generated by gel. This is useful for CI pipelines.

```bash
gelx check
```

If there are changes that haven't been accounted for, the check will fail and you should regenerate the code.

## Future Work

This crate is still in early development and there are several features that are not yet implemented.

### Missing Types

Currently the following types are not supported:

- `MultiRange` - The macro will panic if a multirange is used.

#### `MultiRange`

These are not currently exported by the `gel-protocol` so should be added in a PR to the `gel-protocol` crate, if they are still supported in the new protocol.

### LSP parsing

Currently the macro depends on having a running gel instance to parse the query string.

Once an LSP is created for gel it would make sense to switch from using string to using inline gel queries.

```rust,ignore
use gelx::gelx;

gelx!(
	example,
	select User {**}
);
```

[crate-image]: https://img.shields.io/crates/v/gelx.svg
[crate-link]: https://crates.io/crates/gelx
[docs-image]: https://docs.rs/gelx/badge.svg
[docs-link]: https://docs.rs/gelx/
[ci-status-image]: https://github.com/ifiokjr/gelx/workflows/ci/badge.svg
[ci-status-link]: https://github.com/ifiokjr/gelx/actions?query=workflow:ci
[unlicense-image]: https://img.shields.io/badge/license-Unlicense-blue.svg
[unlicense-link]: https://opensource.org/license/unlicense
[codecov-image]: https://codecov.io/github/ifiokjr/gelx/graph/badge.svg?token=87K799Q78I
[codecov-link]: https://codecov.io/github/ifiokjr/gelx

## Features

- **`default`** — The default feature is `with_all`.
- **`with_bigint`** — Include the `num-bigint` dependency.
- **`with_bigdecimal`** — Use the `bigdecimal` crate.
- **`with_chrono`** — Use the `chrono` crate for all dates.
- **`with_all`** _(enabled by default)_ — Include all additional types. This is included by default. Use `default-features = false` to disable.
- **`builder`** — Use the `typed-builder` crate to generate the builders for the generated `Input` structs.
- **`query`** — Turn on the `query` and `transaction` methods and anything that relies on `gel-tokio`. The reason to separate this feature is to enable usage of this macro in browser environments where `gel-tokio` is not feasible.
- **`serde`** — Enable serde for the generated code.

## Contributing

[`devenv`](https://devenv.sh/) is used to provide a reproducible development environment for this project. Follow the [getting started instructions](https://devenv.sh/getting-started/).

To automatically load the environment you should [install direnv](https://devenv.sh/automatic-shell-activation/) and then load the `direnv`.

```bash
# The security mechanism didn't allow to load the `.envrc`.
# Since we trust it, let's allow it execution.
direnv allow .
```

At this point you should see the `nix` commands available in your terminal.

Run the following commands to install all the required dependencies.

```bash
install:all
```

This installs all the cargo binaries locally so you don't need to worry about polluting your global namespace.

At this point you must setup the gel instance.

```bash
db:setup # setup the gel instance
```

Now you can make your changes and run tests.

```bash
test:all
```

### Available Commands

- `build:all`: Build all crates with all features activated.
- `build:docs`: Build documentation site.
- `coverage:all`: Test all files and generate a coverage report for upload to codecov.
- `db:destroy`: Destroy the local database.
- `db:setup`: Setup the local database.
- `db:up`: Watch changes to the local database.
- `fix:all`: Fix all fixable lint issues.
- `fix:clippy`: Fix fixable lint issues raised by rust clippy.
- `fix:format`: Fix formatting for entire project.
- `install:all`: Install all dependencies.
- `install:cargo:bin`: Install cargo binaries locally.
- `lint:all`: Lint all project files.
- `lint:clippy`: Check rust clippy lints.
- `lint:format`: Check all formatting is correct.
- `setup:ci`: Setup the GitHub ci environment.
- `setup:helix`: Setup the helix editor for development.
- `setup:vscode`: Setup the vscode editor for development.
- `test:all`: Test all project files.
- `update:deps`: Update dependencies.

### Upgrading `devenv`

If you have an outdated version of `devenv` you can update it by running the following commands. If you have an easier way, please create a PR and I'll update these docs.

```bash
nix profile list # find the index of the devenv package
nix profile remove <index>
nix profile install ---accept-flake-config nixpkgs#devenv
```

### Editor Setup

To setup recommended configuration for your favorite editor run the following commands.

```bash
setup:vscode # Setup vscode with recommended configuration
```

```bash
setup:helix # Setup helix with recommended configuration
```

## License

Unlicense, see the [license](./license) file.
