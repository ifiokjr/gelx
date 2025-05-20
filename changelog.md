# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/ifiokjr/gelx/compare/v0.4.0...v0.4.1) - 2025-05-20

### <!-- 0 -->🎉 Added

- *(gelx)* `gelx!` can now take a custom path to the query

### <!-- 1 -->🐛 Bug Fixes

- *(gelx)* update deprecation notice for `gelx_file!` macro

## [0.4.0](https://github.com/ifiokjr/gelx/compare/v0.3.0...v0.4.0) - 2025-05-19

### <!-- 0 -->🎉 Added

- [**breaking**] code the CLI for generating and checking code
- add `GelxMetadata` and improve everything
- add strum support
- add `regex` dependency and refactor code generation functions
- add `gelx_cli` crate and integrate `clap` for command-line interface

### <!-- 3 -->📚 Documentation

- update readme.md
- small improvements

### <!-- 6 -->🧪 Testing

- fix broken doc tests
- update codegen files
- update snapshots

### <!-- 7 -->⚙️ Miscellaneous Tasks

- remove `test_no_features` due to issues

## [0.3.0](https://github.com/ifiokjr/gelx/compare/v0.2.1...v0.3.0) - 2025-05-16

### <!-- 0 -->🎉 Added
- [**breaking**] rename `edgedb_codegen` to `gelx`
- [**breaking**] rename `edgedb_codegen_macros` to `gelx_macros`
- [**breaking**] rename `edgedb_codegen_core` to `gelx_core`
- [**breaking**] upgrade all `gel` libraries to latest versions as a replacement for previous `edgedb` dependencies

### <!-- 3 -->📚 Documentation
- improve main readme

## [0.2.1](https://github.com/ifiokjr/gelx/compare/v0.2.0...v0.2.1) - 2024-08-29

### <!-- 3 -->📚 Documentation
- improve contributing section
- add feature docs for `docs.rs`

## [0.2.0](https://github.com/ifiokjr/gelx/compare/v0.1.2...v0.2.0) - 2024-08-28

### <!-- 0 -->🎉 Added
- [**breaking**] add optional `builder` feature limited to `Input`

### <!-- 3 -->📚 Documentation
- reorder package fields

### <!-- 6 -->🧪 Testing
- improve coverage

## [0.1.2](https://github.com/ifiokjr/gelx/compare/v0.1.1...v0.1.2) - 2024-08-27

### <!-- 3 -->📚 Documentation
- add coverage badge

### <!-- 6 -->🧪 Testing
- update to use `prettyplease` in snapshots
- add coverage support

### <!-- 7 -->⚙️ Miscellaneous Tasks
- support testing multiple rust versions

## [0.1.1](https://github.com/ifiokjr/gelx/compare/v0.1.0...v0.1.1) - 2024-08-26

### <!-- 0 -->🎉 Added
- add support for `Range` types

### <!-- 1 -->🐛 Bug Fixes
- make `gel-derive` optional

### <!-- 3 -->📚 Documentation
- update readme
- add `cli` to future work
- show where shared types should be placed
- explain future work

### <!-- 6 -->🧪 Testing
- `Range` not `Queryable` so needs upstream support

### <!-- 7 -->⚙️ Miscellaneous Tasks
- improve changelog generation

## [0.1.0](https://github.com/ifiokjr/gelx/releases/tag/gelx-v0.1.0) - 2024-08-25

### 🎉 Added

- Initial release
