# `gelx_build`

<br />

> By default the `gelx` macros can't read the configuration from the `Cargo.toml` file. This crate provides a way to read the configuration from the `Cargo.toml` file using the build.rs script.

<br />

[![Crate][crate-image]][crate-link] [![Docs][docs-image]][docs-link] [![Status][ci-status-image]][ci-status-link] [![Unlicense][unlicense-image]][unlicense-link] [![codecov][codecov-image]][codecov-link]

## Overview

The `gelx_build` crate provides a way to read the configuration from the `Cargo.toml` file using the `build.rs` script.

This crate is only needed if you want to customise the configuration of the `gelx` macros.

## Installation

```toml
[build-dependencies]
gelx_build = "0.8"
```

or via the command line:

```bash
cargo add --build gelx_build
```

## Usage

```rust
// build.rs

use gelx_build::gelx_build_sync;

fn main() {
	let _ = gelx_build_sync();
}
```

The above code will read the configuration from the `Cargo.toml` file and create an environment variable called `GELX_METADATA_BASE64` that contains the json configuration. The `GELX_METADATA_BASE64` environment variable is then used by the `gelx_macros` crate to read the configuration and use it when generating code.

If you would like to use the async version, you can use the `gelx_build` function instead.

```rust
// build.rs

use gelx_build::gelx_build;
use gelx_build::tokio;

#[tokio::main]
async fn main() {
	let _ = gelx_build().await;
}
```

## Contributing

This crate is part of the `gelx` workspace. Please refer to the [main project\'s contributing guide](https://github.com/ifiokjr/gelx/blob/main/readme.md#contributing) for details on how to set up the development environment and contribute.

## License

Unlicense, see the [license file](https://github.com/ifiokjr/gelx/blob/main/license) in the root of the workspace.

[crate-image]: https://img.shields.io/crates/v/gelx_build.svg
[crate-link]: https://crates.io/crates/gelx_build
[docs-image]: https://docs.rs/gelx_build/badge.svg
[docs-link]: https://docs.rs/gelx_build/
[ci-status-image]: https://github.com/ifiokjr/gelx/workflows/ci/badge.svg?branch=main
[ci-status-link]: https://github.com/ifiokjr/gelx/actions?query=workflow%3Aci+branch%3Amain
[unlicense-image]: https://img.shields.io/badge/license-Unlicense-blue.svg
[unlicense-link]: https://github.com/ifiokjr/gelx/blob/main/LICENSE
[codecov-image]: https://codecov.io/github/ifiokjr/gelx/graph/badge.svg?token=87K799Q78I
[codecov-link]: https://codecov.io/github/ifiokjr/gelx
