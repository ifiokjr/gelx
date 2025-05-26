# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0](https://github.com/ifiokjr/gelx/compare/v0.6.0...v0.7.0) - 2025-05-26

### <!-- 0 -->🎉 Added

- feat!(gelx_build): add `gelx_build` crate for reading configuration in macros
- add support for postgis via `geo`

### <!-- 1 -->🐛 Bug Fixes

- *(gelx_core)* remove unused `gelx_core` `with_*` features

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

## [0.4.0](https://github.com/ifiokjr/gelx/compare/v0.3.0...v0.4.0) - 2025-05-19

### <!-- 0 -->🎉 Added

- add `GelxMetadata` and improve everything
- add strum support
- add `regex` dependency and refactor code generation functions
- add `gelx_cli` crate and integrate `clap` for command-line interface

### <!-- 3 -->📚 Documentation

- small improvements

## [0.3.0](https://github.com/ifiokjr/gelx/compare/v0.2.1...v0.3.0) - 2025-05-16

### <!-- 0 -->🎉 Added

- [**breaking**] rename `edgedb_codegen` to `gelx`
- [**breaking**] rename `edgedb_codegen_macros` to `gelx_macros`
- [**breaking**] rename `edgedb_codegen_core` to `gelx_core`
- [**breaking**] upgrade all `gel` libraries to latest versions as a replacement for previous `edgedb` dependencies

### <!-- 3 -->📚 Documentation

- improve main readme

## [0.2.0](https://github.com/ifiokjr/gelx/compare/v0.1.2...v0.2.0) - 2024-08-28

### <!-- 0 -->🎉 Added

- [**breaking**] add optional `builder` feature limited to `Input`

### <!-- 3 -->📚 Documentation

- reorder package fields

## [0.1.1](https://github.com/ifiokjr/gelx/compare/0.1.0...0.1.1) - 2024-08-26

### <!-- 7 -->⚙️ Miscellaneous Tasks

- improve changelog generation

## [0.1.0](https://github.com/ifiokjr/gelx/releases/tag/gelx_macros-v0.1.0) - 2024-08-25

### 🎉 Added

- Initial release
