use near_verify_rs::types::contract_source_metadata::ContractSourceMetadata;

struct TestCase {
    input: &'static str,
    output: &'static str,
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
        test_case.output,
        "Artifact hash-sum mismatch"
    );

    Ok(())
}

/// https://testnet.nearblocks.io/address/simple-package-verify-rs-ci.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-package-v1.0.0
const SIMPLE_PACKAGE_EXPECTED: TestCase = TestCase {
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
    output: "5KaX9FM9NtjpfahksL8TMWQk3LF7k8Sv88Qem4tGrVDW",
};

#[test]
fn test_simple_package_vanilla() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_PACKAGE_EXPECTED)?;
    Ok(())
}

mod checkout;
