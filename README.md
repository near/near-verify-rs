# near-verify-rs

[![crates.io](https://img.shields.io/crates/v/near-verify-rs.svg)](https://crates.io/crates/near-verify-rs)
[![Documentation](https://docs.rs/near-verify-rs/badge.svg)](https://docs.rs/near-verify-rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/near-verify-rs.svg)](https://github.com/near/near-verify-rs#license)

Reference implementation of [NEP-330](https://github.com/near/NEPs/blob/master/neps/nep-0330.md) 1.2.0+ for reproducible NEAR Protocol smart contract builds.

## Overview

`near-verify-rs` is a Rust library that enables verification of NEAR Protocol smart contracts through reproducible builds. It compiles smart contracts using Docker containers based on metadata specified in the NEP-330 format, ensuring that the build process can be verified and reproduced by anyone.

## Use Case

When you deploy a NEAR smart contract, users and auditors may want to verify that the deployed WASM matches the claimed source code. This library helps you:

1. **Compile** NEAR Protocol smart contracts reproducibly using Docker
2. **Verify** that a deployed contract matches its source code
3. **Compute** checksums of build artifacts to compare with deployed contracts

The main workflow involves:
- Reading contract source metadata (NEP-330 format)
- Pulling the specified Docker image
- Executing the build command inside the Docker container
- Computing the hash of the resulting WASM file

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
near-verify-rs = "0.3"
```

## Quick Start

Here's a basic example of how to use `near-verify-rs` to verify a NEAR smart contract:

```rust
use near_verify_rs::types::contract_source_metadata::ContractSourceMetadata;
use near_verify_rs::types::source_id::SourceId;
use near_verify_rs::logic;
use camino::Utf8PathBuf;

// Parse the contract metadata (NEP-330 format)
let metadata_json = r#"{
  "build_info": {
    "build_command": ["cargo", "near", "build", "non-reproducible-wasm", "--locked"],
    "build_environment": "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2",
    "contract_path": "",
    "source_code_snapshot": "git+https://github.com/example/contract?rev=abc123"
  },
  "link": "https://github.com/example/contract/tree/abc123",
  "standards": [{"standard": "nep330", "version": "1.2.0"}],
  "version": "1.0.0"
}"#;

let contract_metadata: ContractSourceMetadata = serde_json::from_str(metadata_json)?;

// Extract the source code location
let source_id = SourceId::from_url(
    &contract_metadata.build_info.as_ref().unwrap().source_code_snapshot
)?;

// Checkout the source code to a directory
// (You'll need to implement this using git2 or similar - see tests/checkout.rs for an example)
let target_dir = Utf8PathBuf::from("/path/to/checked/out/source");

// Validate the metadata
contract_metadata.validate(None)?;

// Run the reproducible build in Docker
let wasm_path = logic::nep330_build::run(
    contract_metadata,
    target_dir,
    vec![],  // additional Docker run arguments (e.g., ["--network=host"])
    false,   // quiet mode
)?;

// Compute the hash of the built WASM
let checksum = logic::compute_hash(wasm_path)?;
println!("Contract hash: {}", checksum.to_base58_string());
```

## Features

- **NEP-330 Support**: Full implementation of NEP-330 standard versions 1.2.0+
- **Docker Integration**: Automated Docker container management for reproducible builds
- **Hash Computation**: SHA-256 checksum generation with Base58 encoding
- **Source Code Handling**: Support for Git repositories and other source locations
- **Validation**: Metadata validation with optional whitelisting

## NEP-330 Metadata

The library expects contract metadata in the NEP-330 format, which includes:

- `build_info.build_command`: The command to execute inside the Docker container
- `build_info.build_environment`: Docker image with SHA256 digest
- `build_info.source_code_snapshot`: URL to the source code (e.g., `git+https://...`)
- `build_info.contract_path`: Optional path to the contract within the repository
- `build_info.output_wasm_path`: Optional explicit path to the output WASM file

## Documentation

For more information about NEP-330 and contract source metadata:
- [NEP-330 Specification](https://github.com/near/NEPs/blob/master/neps/nep-0330.md)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
