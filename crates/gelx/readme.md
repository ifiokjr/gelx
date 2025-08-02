<p align="center">
  <a href="#">
    <img width="300" src="https://raw.githubusercontent.com/ifiokjr/gelx/main/setup/assets/logo.svg"  />
  </a>
</p>

<br />

> Generate fully typed rust code from your gel schema and inline queries with `gelx`.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Installation

To install the `gelx` crate, you can use the following commands.

```bash
cargo add gelx
```

Or, add the following directly to your `Cargo.toml` file.

```toml
[dependencies]
gelx = "0.8"
```

Make sure you've [installed](https://docs.geldata.com/reference/using/cli#ref-cli-gel-install) the `gel` CLI for your platform.

Once installed you should be able to run the following command to verify your installation.

```bash
gel --version
```

And then initialize in the project directory.

```bash
gel init
```

## `gelx!`

Working with the default `gel` crate requires manually writing untyped queries and creating the rust structs and enums for both the input and output into these queries with. The correctness of your code can only be checked at runtime increasing the risk of bugs and errors.

`gelx!` transforms your queries into rust structs, types and functions, providing safety during development of your project.

### Inline Queries

```rust
use gel_errors::Error;
use gel_tokio::create_client;
use gelx::gelx;

// Creates a module called `example` with a function called `query` and structs
// for the `Input` and `Output`.
gelx!(
	example,
	"select { hello := \"world\", custom := <str>$custom }"
);

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = create_client().await?;
	let input = example::Input {
		custom: String::from("custom"),
	};

	// For queries the following code can be used.
	let output = example::query(&client, &input).await?;

	Ok(())
}
```

The macro above generates the following code in the background:

```rust
pub mod example {
	use ::gelx::exports as __g;
	/// Execute the desired query.
	pub async fn query(
		client: &__g::gel_tokio::Client,
		props: &Input,
	) -> ::core::result::Result<Output, __g::gel_errors::Error> {
		client.query_required_single(QUERY, props).await
	}
	/// Compose the query as part of a larger transaction.
	pub async fn transaction(
		conn: &mut __g::gel_tokio::Transaction,
		props: &Input,
	) -> ::core::result::Result<Output, __g::gel_errors::Error> {
		conn.query_required_single(QUERY, props).await
	}
	#[derive(
		Clone,
		Debug,
		__g::serde::Serialize,
		__g::serde::Deserialize,
		__g::typed_builder::TypedBuilder,
		__g::gel_derive::Queryable,
	)]
	pub struct Input {
		#[builder(setter(into))]
		pub custom: String,
	}
	impl __g::gel_protocol::query_arg::QueryArgs for Input {
		fn encode(
			&self,
			encoder: &mut __g::gel_protocol::query_arg::Encoder,
		) -> core::result::Result<(), __g::gel_errors::Error> {
			let map = __g::gel_protocol::named_args! {
				"custom" => self.custom.clone(),
			};
			map.encode(encoder)
		}
	}
	#[derive(
		Clone, Debug, __g::serde::Serialize, __g::serde::Deserialize, __g::gel_derive::Queryable,
	)]
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
use gelx::create_client;
use gelx::gelx;

// Creates a module called `select_user` with public functions `transaction` and
// `query` as well as structs for the `Input` and `Output`.
gelx!(select_user);

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

### `gelx_build`

By default, macros can't read from the `Cargo.toml` file of the consuming crate. The `gelx_build` crate provides a way to read the configuration from the `Cargo.toml` file using the `build.rs` script.

You can read the [gelx_build readme](https://github.com/ifiokjr/gelx/blob/main/crates/gelx_build/readme.md) for more information.

## CLI

The `gelx_cli` crate exposes a binary called `gelx` transforms the typed code into `*.rs` files rather than inline queries.

It should be run from the crate directory and will read from the configuration specified in the next section.

```bash
gelx generate --cwd path/to/crate
```

Sometimes you will need to check that the generated code matches the current database schema and queries generated by `gel`. This is useful for CI pipelines to ensure the generated code is up to date.

```bash
gelx check --cwd path/to/crate
```

If there are changes that haven't been accounted for, the check will fail with a diff and you should regenerate the code.

More information can be found in the [`gelx_cli` readme](https://github.com/ifiokjr/gelx/blob/main/crates/gelx_cli/readme.md).

### Globals

The `gelx_cli` will generate a `Globals` struct for your project. It iterates over all the `schema::Global` types defined in your `.gel` schema to generate a `Globals` struct. This struct can be used to create a gel client.

For example if you have the following globals in your schema.

```edgeql
module default {
	global current_user_id: uuid;
	global current_user := (
  	select User filter .id = global current_user_id
	);
	global alternative: str;
}
```

The generated `Globals` struct will look like the following. Notice how the `current_user` global, which is an alias type, is ignored. This is because it can't be set externally and is automatically derived from the `current_user_id` global (it only exists within the database).

```rust,ignore
// src/db/mod.rs
use ::gelx::exports as __g;
#[derive(
	::std::fmt::Debug,
	::core::clone::Clone,
	__g::serde::Serialize,
	__g::serde::Deserialize,
	__g::typed_builder::TypedBuilder,
)]
#[cfg_attr(feature = "ssr", derive(__g::gel_derive::Queryable))]
#[builder(crate_module_path = __g::typed_builder)]
#[builder(field_defaults(default, setter(into, strip_option(fallback_suffix = "_opt"))))]
pub struct Globals {
	pub alternative: Option<String>,
	pub current_user_id: Option<__g::uuid::Uuid>,
}
#[cfg(feature = "ssr")]
impl __g::gel_tokio::GlobalsDelta for Globals {
	fn apply(self, modifier: &mut __g::gel_tokio::state::GlobalsModifier<'_>) {
		modifier.set("additional::alternative", self.alternative);
		modifier.set("default::current_user_id", self.current_user_id);
	}
}
impl Globals {
	/// Create a gel client with the globals.
	pub async fn into_client(
		self,
	) -> ::core::result::Result<__g::gel_tokio::Client, __g::gel_tokio::Error> {
		let client = __g::gel_tokio::create_client().await?.with_globals(self);
		Ok(client)
	}

	/// Create a gel client with the globals.
	pub async fn to_client(
		&self,
	) -> ::core::result::Result<__g::gel_tokio::Client, __g::gel_tokio::Error> {
		let client = self.clone().into_client().await?;
		Ok(client)
	}
}
```

The above code can be used to create a gel client with the globals.

```rust,ignore
use crate::db::Globals;
use gelx::exports::uuid::Uuid;

// Using the builder pattern
let client = Globals::builder()
	.current_user_id(Uuid::new_v4())
	.alternative("test")
	.build()
	.into_client()
	.await?;

// Using the `to_client` method
let client = Globals {
	current_user_id: Some(Uuid::new_v4()),
	alternative: Some("test".to_string()),
}.into_client().await?;
```

## Configuration

The following configuration options are supported. The provided defaults will be used if a value is not specified.

```toml
[package.metadata.gelx]
# The path to the directory containing the queries.
queries_path = "./queries"

# The features to enable and their aliases. By default all features are enabled.
# To disable a feature set its value to `false`. To alias a feature behind a
# feature flag use the following format `feature = { query = "ssr" }`. This will
# enable the query feature only when the `ssr` feature is enabled.
#
# The available features are:
#
# - `query` - When enabled you must include `gel-protocol` as a dependency.
# - `serde` - Enable `serde` for the generated code.
features = { query = true, serde = true }

# The location of the generated code when using the `gelx` CLI.
output_path = "./src/db"

# The name of the arguments input struct. Will be transformed to PascalCase.
input_struct_name = "Input"

# The name of the exported output struct for generated queries. Will be transformed to PascalCase.
output_struct_name = "Output"

# The name of the query function exported.
query_function_name = "query"

# The name of the transaction function exported.
transaction_function_name = "transaction"

# The name of the query constant exported.
query_constant_name = "QUERY"

# The alias used for the `gelx::exports` module.
exports_alias = "__g"

# The macros which are always derived for the generated structs.
struct_derive_macros = [
	"::std::fmt::Debug",
	"::core::clone::Clone",
]

# The macros which are always derived for the generated scalar types which are wrapper structs.
# As envisiaged by <https://github.com/ifiokjr/gelx/issues/29>
scalar_derive_macros = [
	"::std::fmt::Debug",
	"::core::clone::Clone",
]

# The macros which are always derived for the generated enums.
enum_derive_macros = [
	"::std::fmt::Debug",
	"::core::clone::Clone",
	"::core::marker::Copy",
]

# The relative path to the `gel` config file. This is optional, and if not
# provided, the `gel` config will be read from the environment variables.
gel_config_path = "./gel.toml"

# The name of the `gel` instance to use. This is optional, and if not provided,
# the environment variable `$GEL_INSTANCE` will be used.
gel_instance = "$GEL_INSTANCE"

# The name of the `gel` branch to use. This is optional, and if not provided,
# the environment variable `$GEL_BRANCH` will be used.
gel_branch = "$GEL_BRANCH"
```

## `Geometry` and `Geography`

The `gelx` crate provides wrapper types for the `Geometry` and `Geography` types from the `geo` crate.

```edgeql
# queries/insert_location.edgeql

with NewLocation := (insert Location {
	point := <ext::postgis::geometry>$point,
	area := <ext::postgis::geography>$area,
})
select NewLocation {
	point,
	area,
};
```

```rust
use gel_errors::Error;
use gelx::Geography;
use gelx::Geometry;
use gelx::create_client;
use gelx::gelx;
use gelx::geo::point;
use gelx::geo::polygon;

// Creates a module called `insert_location` with public functions `transaction`
// and `query` as well as structs for the `Input` and `Output`.
gelx!(insert_location);

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = create_client().await?;
	let point = point!(x: 1.0, y: 1.0);
	let polygon = polygon![
		(x: -111., y: 45.),
		(x: -111., y: 41.),
		(x: -104., y: 41.),
		(x: -104., y: 45.),
	];
	let output = insert_location::query(
		&client,
		&insert_location::Input {
			point: Geometry(point.into()),
			area: Geography(polygon.into()),
		},
	)
	.await?;

	println!("{:?}", output);

	Ok(())
}
```

## Missing Types

The following types are not currently supported:

- `MultiRange` - The CLI/macro will panic if a multirange is used.

#### `MultiRange`

These are not currently exported by `gel-protocol` and should be added in a PR to the `gel-protocol` crate if they are still supported in the new protocol.

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

## Crate Features

- **`default`** — The default feature is `with_all`.
- **`with_bigint`** — Include the `num-bigint` dependency.
- **`with_bigdecimal`** — Use the `bigdecimal` crate.
- **`with_chrono`** — Use the `chrono` crate for all dates.
- **`with_geo`** — Use the `geo` crate for all geometry and geography types.
- **`with_all`** _(enabled by default)_ — Include all additional types. This is included by default. Use `default-features = false` to disable.
- **`builder`** — Use the `typed-builder` crate to generate the builders for the generated `Input` structs.
- **`query`** — Turn on the `query` and `transaction` methods and anything that relies on `gel-tokio`. The reason to separate this feature is to enable usage of this macro in browser environments where `gel-tokio` is not feasible.
- **`serde`** — Enable `serde` for the generated code.
- **`strum`** - Use the `strum` crate for deriving strings from the created enums.

## Recommended Setup

Create a `gel.toml` in the root of your project with the following configuration. The following configuration will work for a single crate project.

```toml
[instance]
server-version = "6.7"

[project]
schema-dir = "dbschema"

[hooks]
project.init.after = "gelx generate"
branch.switch.after = "gelx generate"
schema.update.after = "gelx generate"

[[watch]]
files = ["dbschema/*.gel"]
script = "gelx generate"

[[watch]]
files = ["queries/*.edgeql"]
script = "gelx generate"
```

By default this will generate the code into the `src/db` directory. You can change this by setting the `output_path` in the configuration.

## Contributing

[`devenv`](https://devenv.sh/) is used to provide a reproducible development environment for this project. Follow the [getting started instructions](https://devenv.sh/getting-started/).

To automatically load the environment you should [install direnv](https://devenv.sh/automatic-shell-activation/) and then run `direnv allow .`.

```bash
direnv allow .
```

You now have a shell with all the dependencies installed and project-specific commands available.

Run the following commands to install all the required dependencies.

```bash
install:all
```

This installs all the cargo binaries locally so you don't need to worry about polluting your global namespace.

At this point you must setup the `gel` instance.

```bash
db:setup # setup the gel instance
```

The above command will setup the local database and install the postgis extension.

Now you can make your changes and run tests.

```bash
test:all
```

### Available Commands

You can view all the available scripts, packages, tasks and environment variables by running `devenv info`.

- `build:all`: Build all crates with all features activated.
- `build:docs`: Build documentation site.
- `coverage:all`: Test all files and generate a coverage report for upload to codecov.
- `db:destroy`: Destroy the local database.
- `db:reset`: Reset the local database.
- `db:setup`: Setup the local database.
- `db:up`: Watch changes to the local database.
- `fix:all`: Fix all fixable lint issues.
- `fix:clippy`: Fix fixable lint issues raised by rust clippy.
- `fix:format`: Fix formatting for entire project.
- `fix:gelx`: Fix fixable lint issues raised by gelx.
- `install:all`: Install all dependencies.
- `install:cargo:bin`: Install cargo binaries locally.
- `lint:all`: Lint all project files.
- `lint:clippy`: Check rust clippy lints.
- `lint:format`: Check all formatting is correct.
- `lint:gelx`: Check gelx is formatted correctly.
- `test:all`: Test all project files.
- `update:deps`: Update all project dependencies.

### Upgrading `devenv`

If you have an outdated version of `devenv`, you can update it by running the following commands. If you know of an easier way, please create a PR to update these docs.

```bash
nix profile list # find the index of the devenv package
nix profile remove <index>
nix profile install ---accept-flake-config nixpkgs#devenv
```

### Editor Setup

To setup recommended configuration for your favorite editor run the corresponding command.

```bash
setup:vscode # Setup vscode with recommended configuration
```

```bash
setup:helix # Setup helix with recommended configuration
```

## License

Unlicense, see the [license](https://github.com/ifiokjr/gelx/blob/main/license) file.
