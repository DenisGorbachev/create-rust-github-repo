# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.2](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.6.1...v0.6.2) - 2025-04-11

### Fixed

- lint

## [0.6.1](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.6.0...v0.6.1) - 2025-04-10

### Other

- update Cargo.lock dependencies

## [0.6.0](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.8...v0.6.0) - 2025-02-22

### Fixed

- [**breaking**] rename args to cmd

## [0.5.8](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.7...v0.5.8) - 2025-02-17

### Other

- update README

## [0.5.7](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.6...v0.5.7) - 2025-02-16

### Other

- fix remark

## [0.5.6](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.5...v0.5.6) - 2025-02-15

### Fixed

- update README

## [0.5.5](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.4...v0.5.5) - 2025-01-27

### Added

- after_all_cmd
- make test_support_link async
- add hedgehog-rs dependency

### Fixed

- Use std::process::ExitStatus and remove unused import of Child
- Improve spawn_and_wait and is_success methods in Shell and Executor
- Resolve compilation errors in create-rust-github-repo

### Other

- update mise version
- Merge branch 'main' of github.com:DenisGorbachev/create-rust-github-repo
- Use tokio Command instead of std Command
- Refactor fn main in every bin and fn run to use async tokio. Keep stdout and stderr. Note that display_message_box call passes stderr as writer argument.
- update deps

## [0.5.4](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.3...v0.5.4) - 2024-08-26

### Added
- add create-rust-keybase-private-lib

### Other
- sort deps

## [0.5.3](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.2...v0.5.3) - 2024-08-05

### Fixed
- commitlint
- commitlint

### Other
- use better checkmarks
- use the star link
- allow more types in commitlint

## [0.5.2](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.1...v0.5.2) - 2024-07-30

### Fixed
- copy directories recursively

## [0.5.1](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.5.0...v0.5.1) - 2024-07-29

### Fixed
- use conventional commit message

### Other
- fix caching
- improve caching

## [0.5.0](https://github.com/DenisGorbachev/create-rust-github-repo/compare/v0.4.0...v0.5.0) - 2024-07-24

### Added
- [**breaking**] add --dry-run, add command output, add shell arguments, implement the support message

### Other
- install cargo-machete

## [0.3.4](https://github.com/DenisGorbachev/create-rust-github-repo/releases/tag/v0.3.4) - 2024-07-21

### Other

- add release-plz
- fix parsing
