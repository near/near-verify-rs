# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.4](https://github.com/near/near-verify-rs/compare/v0.3.3...v0.3.4) - 2026-05-20

### Other

- trusted publishing to crates.io via release-plz + OIDC ([#23](https://github.com/near/near-verify-rs/pull/23))

## [0.3.3](https://github.com/near/near-verify-rs/compare/v0.3.2...v0.3.3) - 2026-04-15

### Fixed

- Append SELinux shared relabel for the bind mount (:z) ([#22](https://github.com/near/near-verify-rs/pull/22))

### Other

- upgrade to Rust edition 2024 ([#20](https://github.com/near/near-verify-rs/pull/20))

## [0.3.2](https://github.com/near/near-verify-rs/compare/v0.3.1...v0.3.2) - 2026-01-13

### Other

- update CODEOWNERS to reflect new ownership ([#19](https://github.com/near/near-verify-rs/pull/19))
- update `cargo_metadata` dependency to enbale building with 1.86 ([#18](https://github.com/near/near-verify-rs/pull/18))
- Add README and LICENSE files ([#16](https://github.com/near/near-verify-rs/pull/16))

## [0.3.1](https://github.com/near/near-verify-rs/compare/v0.3.0...v0.3.1) - 2025-05-29

### Other

- update snapshots to released (docker image, `cargo-near-build` package) ([#14](https://github.com/near/near-verify-rs/pull/14))
- test [cargo_near_build::extended::build_with_cli] ([#13](https://github.com/near/near-verify-rs/pull/13))
- factory with build-scripts, running binary `cargo-near` (no lib) ([#10](https://github.com/near/near-verify-rs/pull/10))
- Upgraded dependencies to the latest releases
- Updated the CODEOWNERS

## [0.3.0](https://github.com/near/near-verify-rs/compare/v0.2.1...v0.3.0) - 2025-04-22

### Added

- var changes, small ([#9](https://github.com/near/near-verify-rs/pull/9))

## [0.2.1](https://github.com/near/near-verify-rs/compare/v0.2.0...v0.2.1) - 2025-04-10

### Other

- remove Cargo.lock ([#6](https://github.com/near/near-verify-rs/pull/6))

## [0.2.0](https://github.com/near/near-verify-rs/compare/v0.1.0...v0.2.0) - 2025-04-10

### Added

- verify with added `output_wasm_path` field to metadata ([#4](https://github.com/near/near-verify-rs/pull/4))

### Other

- test reject push
