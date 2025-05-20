# `gelx_core`

<br />

> Core utilities and logic for `gelx` code generation, powering both the `gelx` procedural macro and the `gelx_cli`.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Overview

The `gelx_core` crate is the engine behind the `gelx` ecosystem. It provides the foundational components and functionalities required for:

- Parsing Gel query files (`.edgeql`).
- Connecting to a Gel instance to introspect schemas and query types.
- Translating Gel query descriptions into Rust `TokenStream`s.
- Handling `gelx` configuration from `Cargo.toml` (`[package.metadata.gelx]`).
- Defining common types, error handling, and utility functions used by both `gelx` (the macro crate) and `gelx_cli` (the command-line tool).

This crate is not typically used directly by end-users but serves as an internal library for the other `gelx` tools. If you are looking to generate Rust code from Gel queries, you should use either the [`gelx` macro](https://crates.io/crates/gelx) for inline queries or the [`gelx_cli`](https://crates.io/crates/gelx_cli) for file-based generation.

## Functionality

Key functionalities provided by `gelx_core` include:

- **Descriptor Fetching**: `get_descriptor_sync` and `get_descriptor` functions to fetch query data descriptions from a running Gel instance.
- **Token Stream Generation**: `generate_query_token_stream` which takes a query description and generates the corresponding Rust code (structs for input/output, and query functions).
- **Metadata Handling**: Structures and functions to parse and manage `GelxMetadata` from `Cargo.toml`.
- **Type Mapping**: Logic to map Gel types to appropriate Rust types (e.g., `String`, `Uuid`, `DateTime`, custom enums, and object shapes).
- **Configuration**: Utilities for resolving Gel connection parameters.
- **Error Handling**: Common error types for the `gelx` ecosystem.

## Features

`gelx_core` exposes several features that can be toggled by dependent crates (like `gelx` and `gelx_macros`):

- `with_bigint`: Enables support for `BigInt`.
- `with_bigdecimal`: Enables support for `BigDecimal`.
- `with_chrono`: Enables support for `chrono` date/time types.
- `with_all`: A convenience feature that enables all `with_*` features.
- `query`: Enables generation of query execution functions (depends on `gel-tokio`).
- `serde`: Enables `#[derive(Serialize, Deserialize)]` for generated structs.
- `builder`: Enables `#[derive(TypedBuilder)]` for generated input structs.
- `strum`: Enables `#[derive(EnumString, Display)]` for generated enums.

These features are typically controlled via the `gelx` crate's own features.

## Contributing

This crate is part of the `gelx` workspace. Please refer to the [main project\'s contributing guide](https://github.com/ifiokjr/gelx/blob/main/readme.md#contributing) for details on how to set up the development environment and contribute.

## License

Unlicense, see the [license file](https://github.com/ifiokjr/gelx/blob/main/license) in the root of the workspace.

[crate-image]: https://img.shields.io/crates/v/gelx_core.svg
[crate-link]: https://crates.io/crates/gelx_core
[docs-image]: https://docs.rs/gelx_core/badge.svg
[docs-link]: https://docs.rs/gelx_core/
[ci-status-image]: https://github.com/ifiokjr/gelx/workflows/ci/badge.svg?branch=main
[ci-status-link]: https://github.com/ifiokjr/gelx/actions?query=workflow%3Aci+branch%3Amain
[unlicense-image]: https://img.shields.io/badge/license-Unlicense-blue.svg
[unlicense-link]: https://github.com/ifiokjr/gelx/blob/main/LICENSE
[codecov-image]: https://codecov.io/github/ifiokjr/gelx/graph/badge.svg?token=87K799Q78I
[codecov-link]: https://codecov.io/github/ifiokjr/gelx
