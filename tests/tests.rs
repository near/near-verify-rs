use near_verify_rs::types::contract_source_metadata::ContractSourceMetadata;
mod checkout;

struct TestCase {
    input: &'static str,
    expected_output: &'static str,
}
fn common_verify_test_routine(test_case: TestCase) -> eyre::Result<()> {
    let contract_source_metadata: ContractSourceMetadata = serde_json::from_str(test_case.input)?;

    assert!(contract_source_metadata.build_info.is_some());
    let source_id = near_verify_rs::types::source_id::SourceId::from_url(
        &contract_source_metadata
            .build_info
            .as_ref()
            .unwrap()
            .source_code_snapshot,
    )?;

    let (_tempdir, target_dir) = checkout::checkout(source_id)?;

    let target_dir = camino::Utf8PathBuf::from_path_buf(target_dir)
        .map_err(|err| eyre::eyre!("convert path buf {:?}", err))?;
    let docker_build_out_wasm =
        near_verify_rs::logic::nep330_build::run(contract_source_metadata, target_dir, vec![])?;

    let result = near_verify_rs::logic::compute_hash(docker_build_out_wasm)?;

    assert_eq!(
        result.to_base58_string(),
        test_case.expected_output,
        "Artifact hash-sum mismatch"
    );

    Ok(())
}

/// https://testnet.nearblocks.io/address/simple-package-verify-rs-ci.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-package-v1.0.0
const SIMPLE_PACKAGE_VANILLA: TestCase = TestCase {
    input: r#"
{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "build_environment": "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2",
    "contract_path": "",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=e3303f0cf8761b99f84f93c3a2d7046be6f4edb5"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/e3303f0cf8761b99f84f93c3a2d7046be6f4edb5",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.0.0"
}"#,
    expected_output: "5KaX9FM9NtjpfahksL8TMWQk3LF7k8Sv88Qem4tGrVDW",
};

#[test]
fn test_simple_package_vanilla() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_PACKAGE_VANILLA)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/simple-package-with-features-verify-rs-ci.testnet
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-package-with-features-v1.0.0
const SIMPLE_PACKAGE_WITH_FEATURES: TestCase = TestCase {
    input: r#"{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked",
      "--no-default-features",
      "--features",
      "near-sdk/legacy"
    ],
    "build_environment": "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2",
    "contract_path": "",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=6fc35ed210d3578b301e25b3b8c11fb53767d032"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/6fc35ed210d3578b301e25b3b8c11fb53767d032",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.0.0"
}"#,
    expected_output: "D5YfnZPCyzdqcdjroW7TGG3GQezdQSrcRWG4mRxdHx5d",
};
#[test]
fn test_simple_package_with_features() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_PACKAGE_WITH_FEATURES)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/simple-package-with-paseed-env-verify-rs-ci.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-package-with-passed-env-v1.0.0
const SIMPLE_PACKAGE_WITH_PASSED_ENV: TestCase = TestCase {
    input: r#"{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked",
      "--env",
      "KEY=VALUE",
      "--env",
      "GOOGLE_QUERY=https://www.google.com/search?q=google+translate&sca_esv=3c150c50f502bc5d"
    ],
    "build_environment": "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2",
    "contract_path": "",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=4f593556476fb0c5d71a73e615a391a972aa586a"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/4f593556476fb0c5d71a73e615a391a972aa586a",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.0.0"
}"#,
    expected_output: "3fdG1ETP7SfArvdfeM9asqNfBj3HKvBK4ZV3uz3eTdzm",
};

#[test]
fn test_simple_package_with_passed_env() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_PACKAGE_WITH_PASSED_ENV)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/simple-factory-verify-rs-cia.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-factory-v1.0.0%2Bsimple-factory-product-v1.1.0
const SIMPLE_FACTORY_VANILLA: TestCase = TestCase {
    input: r#"{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "build_environment": "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2",
    "contract_path": "workspace_root_folder/factory",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=dffdd3a2a33ee3aebb0a72cdccd902f5ab69989c"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/dffdd3a2a33ee3aebb0a72cdccd902f5ab69989c",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.0.0"
}"#,
    expected_output: "7qhDddxfr4p39CeBvpTXWQmzzDA4HTbrWceZtaDAExjW",
};

#[test]
fn test_simple_factory_vanilla() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_VANILLA)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/product.simple-factory-verify-rs-cia.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-factory-v1.0.0%2Bsimple-factory-product-v1.1.0
const SIMPLE_FACTORY_VANILLA_PRODUCT: TestCase = TestCase {
    input: r#"{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "build_environment": "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2",
    "contract_path": "workspace_root_folder/product-donation",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=dffdd3a2a33ee3aebb0a72cdccd902f5ab69989c"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/dffdd3a2a33ee3aebb0a72cdccd902f5ab69989c",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.1.0"
}"#,
    expected_output: "FLXsv6msJ6dD9A6DpJX96d3q8UiDjUtyBsdnEYVnML2U",
};

#[test]
fn test_simple_factory_vanilla_product() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_VANILLA_PRODUCT)?;
    Ok(())
}
