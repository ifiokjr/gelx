# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/ifiokjr/gelx/compare/v0.5.1...v0.6.0) - 2025-05-24

### <!-- 0 -->🎉 Added

- *(gelx)* update config to `output_path` and generate code in folder rather than file
- *(gelx_core)* add derive macros configuration for structs and enums

### <!-- 2 -->🚜 Refactor

- remove `vfs` and complete refactor

### <!-- 6 -->🧪 Testing

- *(gelx)* improve feature testing

## [0.5.0](https://github.com/ifiokjr/gelx/compare/v0.4.0...v0.5.0) - 2025-05-20

### <!-- 0 -->🎉 Added

- _(gelx)_ deprecate `gelx_file` macro
- _(gelx_core)_ add `query_constant_name` metadata option
- _(gelx_core)_ make `exports_alias` customisable
- _(gelx_cli)_ add `--stdout` option for code generation
- _(gelx)_ `gelx!` can now take a custom path to the query

### <!-- 3 -->📚 Documentation

- update installation instructions share readme across crates
- update installation instructions share readme across crates

### <!-- 6 -->🧪 Testing

- _(gelx_cli)_ hardcode `gelx` binary from `devenv` in test

## [0.4.0](https://github.com/ifiokjr/gelx/releases/tag/v0.4.0) - 2025-05-19

### <!-- 0 -->🎉 Added

- _(cli)_ add `--cwd` to set directory for command
- [**breaking**] code the CLI for generating and checking code
- add `GelxMetadata` and improve everything
- add `gelx_example` crate
- add enum generation from schema
- add `regex` dependency and refactor code generation functions
- add `gelx_cli` crate and integrate `clap` for command-line interface

### <!-- 3 -->📚 Documentation

- small improvements

### <!-- 6 -->🧪 Testing

- fix broken doc tests
