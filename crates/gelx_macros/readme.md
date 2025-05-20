# `gelx_macros`

<br />

> Procedural macros for the `gelx` crate, enabling compile-time generation of typed Rust code from Gel queries.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Overview

The `gelx_macros` crate provides the `gelx_raw!` procedural macro, which is the primary way users interact with `gelx` for inline query code generation.

When you use `gelx_raw!(module_name, query: "select ...")` or `gelx_raw!(module_name, file: "path/to/query.edgeql")` in your Rust code, this crate is responsible for:

- Parsing the macro input (module name and query string or file path).
- Invoking `gelx_core` to fetch query descriptions from a Gel instance.
- Utilizing `gelx_core` to generate the Rust `TokenStream` for the query module, including input/output structs and query functions.
- Returning the generated `TokenStream` to the Rust compiler to be included in your crate.

This crate is an internal dependency of the main `gelx` crate and is not intended to be used directly by end-users, other than through the `gelx!` macro re-exported by the `gelx` crate.

## Macro Usage

The main macro provided is `gelx!`. It can be used in two ways:

1. **Inline Query:**
   ```rust
   use gelx::gelx; // Assuming gelx re-exports the macro

   gelx!(my_query_module, "SELECT { message := <str>$arg }");

   // This generates a `my_query_module` with Input/Output structs and query
   // functions.
   ```

2. **File-based Query:**
   ```rust
   use gelx::gelx;

   // Assuming a file `queries/get_user.edgeql` exists relative to your Cargo.toml
   gelx!(get_user_module, file: "queries/get_user.edgeql");

   // Or, if the filename matches the module name (e.g.,
   // `queries/my_module.edgeql`):
   gelx!(my_module);
   ```

For detailed usage examples and configuration, please refer to the documentation of the main [`gelx` crate](https://crates.io/crates/gelx).

## Features

`gelx_macros` forwards its features to `gelx_core`. The available features are:

- `with_bigint`
- `with_bigdecimal`
- `with_chrono`
- `with_all`
- `query`
- `serde`
- `builder`
- `strum`

These features are typically controlled via the `gelx` crate's own features when you add `gelx` as a dependency.

## Contributing

This crate is part of the `gelx` workspace. Please refer to the [main project's contributing guide](https://github.com/ifiokjr/gelx/blob/main/readme.md#contributing) for details on how to set up the development environment and contribute.

## License

Unlicense, see the [license file](https://github.com/ifiokjr/gelx/blob/main/license) in the root of the workspace.

[crate-image]: https://img.shields.io/crates/v/gelx_macros.svg
[crate-link]: https://crates.io/crates/gelx_macros
[docs-image]: https://docs.rs/gelx_macros/badge.svg
[docs-link]: https://docs.rs/gelx_macros/
[ci-status-image]: https://github.com/ifiokjr/gelx/workflows/ci/badge.svg?branch=main
[ci-status-link]: https://github.com/ifiokjr/gelx/actions?query=workflow%3Aci+branch%3Amain
[unlicense-image]: https://img.shields.io/badge/license-Unlicense-blue.svg
[unlicense-link]: https://github.com/ifiokjr/gelx/blob/main/LICENSE
[codecov-image]: https://codecov.io/github/ifiokjr/gelx/graph/badge.svg?token=87K799Q78I
[codecov-link]: https://codecov.io/github/ifiokjr/gelx
