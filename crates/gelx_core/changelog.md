# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/ifiokjr/gelx/compare/gelx_core@v0.3.0...gelx_core@v0.4.0) - 2025-05-19

### <!-- 0 -->🎉 Added

- *(cli)* add `--cwd` to set directory for command
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

## [0.3.0](https://github.com/ifiokjr/gelx/compare/v0.2.1...v0.3.0) - 2025-05-16

### <!-- 0 -->🎉 Added
- [**breaking**] rename `edgedb_codegen` to `gelx`
- [**breaking**] rename `edgedb_codegen_macros` to `gelx_macros`
- [**breaking**] rename `edgedb_codegen_core` to `gelx_core`
- [**breaking**] upgrade all `gel` libraries to latest versions as a replacement for previous `edgedb` dependencies

### <!-- 3 -->📚 Documentation
- improve main readme

## [0.2.0](https://github.com/ifiokjr/gelx/compare/gelx_core@v0.1.2...gelx_core@v0.2.0) - 2024-08-28

### <!-- 0 -->🎉 Added
- [**breaking**] add optional `builder` feature limited to `Input`

### <!-- 3 -->📚 Documentation
- reorder package fields

### <!-- 6 -->🧪 Testing
- improve coverage

## [0.1.2](https://github.com/ifiokjr/gelx/compare/gelx_core@v0.1.1...gelx_core@v0.1.2) - 2024-08-27

### <!-- 0 -->🎉 Added
- *(gelx_core)* add `prettyplease` format option

### <!-- 1 -->🐛 Bug Fixes
- increase minimum rust version

### <!-- 7 -->⚙️ Miscellaneous Tasks
- update coverage files
- support testing multiple rust versions

## [0.1.1](https://github.com/ifiokjr/gelx/compare/gelx_core@0.1.0...gelx_core@0.1.1) - 2024-08-26

### <!-- 0 -->🎉 Added
- add support for `Range` types

### <!-- 7 -->⚙️ Miscellaneous Tasks
- improve changelog generation

## [0.1.0](https://github.com/ifiokjr/gelx/releases/tag/gelx_core-v0.1.0) - 2024-08-25

### 🎉 Added

- Initial release
