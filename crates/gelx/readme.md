# `gelx`

<br />

> Generate fully typed rust code from your EdgeDB schema and inline queries.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Installation

To install the `gelx` crate you can use the following command.

```bash
cargo add gelx
```

Or directly add the following to your `Cargo.toml` file.

```toml
gelx = "0.2"
```

Follow the [Quickstart Guide](https://docs.gel.com/get-started/quickstart) to make sure your gel instance is running. The macro relies on the running `gel` instance to parse the output of the provided query string.

## Usage

When working with `gel` you often need to write queries and also provide the typed for both the input and output. Your code is only checked at runtime which increases the risk of bugs and errors.

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

## Future Work

This crate is still in early development and there are several features that are not yet implemented.

### Missing Types

Currently the following types are not supported:

- `enum` - currently all enums are represented as strings.
- `MultiRange` - The macro will panic if a multirange is used.

#### `enum`

Currently all enums are represented as strings.

In order to support full enum generation the `gel-protocol` crate needs to be [updated](https://github.com/gel/gel-rust/issues/336) to use the [binary protocol 2.0](https://docs.gel.com/database/reference/protocol/typedesc#enumeration-type-descriptor). In the current 1.0 version the enum descriptors are returned without the name property.

Once [this is](https://github.com/gel/gel-rust/issues/336) implemented the macro will be able to generate the correct code.

However end users probably don't want multiple enums for each generated query module as this would break sharing. To get around this, there should be a macro for generating the shared types used by all other.

```rust,ignore
// lib.rs
use gelx::generate_shared_types;

generate_shared_types!(); // exports the shared types to the `edb` module.
```

#### `MultiRange`

These are not currently exported by the `gel-protocol` so should be added in a PR to the `gel-protocol` crate, if they are still supported in the new protocol.

### Configuration

Currently everything is hardcoded and the macro is not configurable.

The following configuration options should be added:

- Name of input struct (optional) - `Input` by default.
- Name of output struct (optional) - `Output` by default.
- Name of query function (optional) - `query` by default.
- Name of transaction function (optional) - `transaction`by default.
- Default location of queries (optional) - `queries` by default.
- Default crate export name for shared types (optional) - `edb` by default.
- Default `gel` instance (optional) - `$GEL_INSTANCE` by default.
- Default `gel` branch (optional) - `$GEL_BRANCH` by default.

Probably these should be read from the `Cargo.toml` file and parsed manually to prevent slowdowns from parsing the file.

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

### CLI

Create a `gelx_cli` crate which supports generating the typed code into rust files rather than inline queries. This is useful for larger projects to prevent constantly compiling the queries on every change / build.

[crate-image]: https://img.shields.io/crates/v/gelx.svg
[crate-link]: https://crates.io/crates/gelx
[docs-image]: https://docs.rs/gelx/badge.svg
[docs-link]: https://docs.rs/gelx/
[ci-status-image]: https://github.com/ifiokjr/gelx/workflows/ci/badge.svg
[ci-status-link]: https://github.com/ifiokjr/gelx/actions?query=workflow:ci
[unlicense-image]: https://img.shields.io/badge/license-Unlicence-blue.svg
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
