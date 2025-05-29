use near_verify_rs::types::{
    contract_source_metadata::ContractSourceMetadata, whitelist::Whitelist,
};
mod checkout;

struct TestCase {
    input: &'static str,
    expected_output: &'static str,
}
fn common_verify_test_routine_opts(
    test_case: TestCase,
    whitelist: Option<Whitelist>,
) -> eyre::Result<()> {
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

    contract_source_metadata.validate(whitelist)?;
    let docker_build_out_wasm = near_verify_rs::logic::nep330_build::run(
        contract_source_metadata,
        target_dir,
        vec![],
        false,
    )?;

    let result = near_verify_rs::logic::compute_hash(docker_build_out_wasm)?;

    assert_eq!(
        result.to_base58_string(),
        test_case.expected_output,
        "Artifact hash-sum mismatch"
    );

    Ok(())
}
fn common_verify_test_routine(test_case: TestCase) -> eyre::Result<()> {
    common_verify_test_routine_opts(test_case, None)
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
fn test_simple_factory_product_vanilla() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_VANILLA_PRODUCT)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/simple-factory-with-features-verify-rs-ci-a.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-factory-with-features-v1.0.0%2Bsimple-factory-product-with-features-v1.1.0
const SIMPLE_FACTORY_WITH_FEATURES: TestCase = TestCase {
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
    "contract_path": "workspace_root_folder/factory",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=0db6242138876e591900d3c0fdac95cc74ac6e89"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/0db6242138876e591900d3c0fdac95cc74ac6e89",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.0.0"
}"#,
    expected_output: "6Nmb4WML7VpKmv8KCJzxMD6SQ1jjhwiVRbKYkx2Jqts1",
};

#[test]
fn test_simple_factory_with_features() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_WITH_FEATURES)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/product.simple-factory-with-features-verify-rs-ci-a.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-factory-with-features-v1.0.0%2Bsimple-factory-product-with-features-v1.1.0
const SIMPLE_FACTORY_WITH_FEATURES_PRODUCT: TestCase = TestCase {
    input: r#"{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked",
      "--features",
      "near-sdk/legacy",
      "--no-default-features"
    ],
    "build_environment": "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2",
    "contract_path": "workspace_root_folder/product-donation",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=0db6242138876e591900d3c0fdac95cc74ac6e89"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/0db6242138876e591900d3c0fdac95cc74ac6e89",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.1.0"
}"#,
    expected_output: "2onZk3T9QqqNTEMwHf6EGBtLUEa4WyebtxDfYzhq5mLW",
};

#[test]
fn test_simple_factory_product_with_features() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_WITH_FEATURES_PRODUCT)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/simple-package-out-path-verify-4-ci.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/simple-pkg-with-out-path-v1.0.0
const SIMPLE_PACKAGE_WITH_OUT_PATH: TestCase = TestCase {
    input: r#"{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "build_environment": "sourcescan/cargo-near:0.14.2-rust-1.86.0@sha256:2320519772d04dd960c2c5c0172c0887ca4407e1c7c04e3be246b07cc5b21db0",
    "contract_path": "",
    "output_wasm_path": "/home/near/code/target/near/simple_package_with_output_path.wasm",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=2a8369686e8793d17925e69e948d1df5f867fdfb"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/2a8369686e8793d17925e69e948d1df5f867fdfb",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "version": "1.0.0"
}"#,
    expected_output: "5t3mTM9gyZaQLCG31qUMZCR6dQNSSSVTMtYP8z43HvUd",
};

#[test]
fn test_simple_package_with_out_path() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_PACKAGE_WITH_OUT_PATH)?;
    Ok(())
}

/// this is a copy of [SIMPLE_PACKAGE_WITH_OUT_PATH] where `target/near`
/// was replaced with `target/bear` in `output_wasm_path`
const SIMPLE_PACKAGE_WITH_WRONG_OUT_PATH: TestCase = TestCase {
    input: r#"{
"build_info": {
  "build_command": [
    "cargo",
    "near",
    "build",
    "non-reproducible-wasm",
    "--locked"
  ],
  "build_environment": "sourcescan/cargo-near:0.14.2-rust-1.86.0@sha256:2320519772d04dd960c2c5c0172c0887ca4407e1c7c04e3be246b07cc5b21db0",
  "contract_path": "",
  "output_wasm_path": "/home/near/code/target/bear/simple_package_with_output_path.wasm",
  "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=2a8369686e8793d17925e69e948d1df5f867fdfb"
},
"link": "https://github.com/dj8yfo/verify_contracts_collection/tree/2a8369686e8793d17925e69e948d1df5f867fdfb",
"standards": [
  {
    "standard": "nep330",
    "version": "1.3.0"
  }
],
"version": "1.0.0"
}"#,
    expected_output: "5t3mTM9gyZaQLCG31qUMZCR6dQNSSSVTMtYP8z43HvUd",
};
#[test]
fn test_simple_package_with_wrong_out_path() -> eyre::Result<()> {
    let Err(err) = common_verify_test_routine(SIMPLE_PACKAGE_WITH_WRONG_OUT_PATH) else {
        panic!("Expecting an error returned from `common_verify_test_routine`");
    };
    println!("{:#?}", err);

    assert!(format!("{:?}", err).contains(
        "assumed artifact result path for a generic nep330 1.3.0 compliant docker build not found"
    ));
    Ok(())
}

/// https://testnet.nearblocks.io/address/simple-factory-with-out-path-b.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/factory-with-out-path-v1.0.0
const SIMPLE_FACTORY_WITH_OUT_PATH: TestCase = TestCase {
    input: r#"{
  "version": "1.0.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/ddf06c68cec61fa8203843f1481de84b38d33b74",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "contract_path": "workspace_root_folder/factory",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=ddf06c68cec61fa8203843f1481de84b38d33b74",
    "output_wasm_path": "/home/near/code/workspace_root_folder/target/near/simple_factory/simple_factory.wasm"
  }
}
"#,
    expected_output: "84msJW47PRRgYqzSNviVjnksbc5E91UyUexuF3AVkv3T",
};

#[test]
fn test_simple_factory_with_out_path() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_WITH_OUT_PATH)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/product.simple-factory-with-out-path-b.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/factory-with-out-path-v1.0.0
const SIMPLE_FACTORY_PRODUCT_WITH_OUT_PATH: TestCase = TestCase {
    input: r#"{
  "version": "1.1.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/ddf06c68cec61fa8203843f1481de84b38d33b74",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "contract_path": "workspace_root_folder/product-donation",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=ddf06c68cec61fa8203843f1481de84b38d33b74",
    "output_wasm_path": "/home/near/code/workspace_root_folder/target/near/simple_factory_product/simple_factory_product.wasm"
  }
}"#,
    expected_output: "GDRdS8giKvhvi2vBrCpbPUTHCuX3zEFxfCswRMdejzco",
};

#[test]
fn test_simple_factory_product_with_out_path() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_PRODUCT_WITH_OUT_PATH)?;
    Ok(())
}
/// https://testnet.nearblocks.io/address/simple-factory-out-path-feat-a.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/factory-with-out-path-features-v1.0.0
const SIMPLE_FACTORY_WITH_OUT_PATH_AND_FEATURES: TestCase = TestCase {
    input: r#"{
  "version": "1.0.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/5530049ceb73e62aff368a631068d0790b2ca6fe",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "contract_path": "workspace_root_folder/factory",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=5530049ceb73e62aff368a631068d0790b2ca6fe",
    "output_wasm_path": "/home/near/code/workspace_root_folder/target/near/simple_factory/simple_factory.wasm"
  }
}"#,
    expected_output: "HdNeKX938emNwL59jLrHF9vc9CT1jVXHRAfhpmXaiFx1",
};

#[test]
fn test_simple_factory_with_out_path_and_features() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_WITH_OUT_PATH_AND_FEATURES)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/product.simple-factory-out-path-feat-a.testnet
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/factory-with-out-path-features-v1.0.0
const SIMPLE_FACTORY_PRODUCT_WITH_OUT_PATH_AND_FEATURES: TestCase = TestCase {
    input: r#"{
  "version": "1.1.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/5530049ceb73e62aff368a631068d0790b2ca6fe",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked",
      "--features",
      "near-sdk/legacy",
      "--no-default-features"
    ],
    "contract_path": "workspace_root_folder/product-donation",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=5530049ceb73e62aff368a631068d0790b2ca6fe",
    "output_wasm_path": "/home/near/code/workspace_root_folder/target/near/simple_factory_product/simple_factory_product.wasm"
  }
}"#,
    expected_output: "AbJNtiZTFi1BnvASBbBR478buVSPtdL1nKQx5ZSsJv8B",
};

#[test]
fn test_simple_factory_product_with_out_path_and_features() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_PRODUCT_WITH_OUT_PATH_AND_FEATURES)?;
    Ok(())
}
/// https://testnet.nearblocks.io/address/discussions.uniquehandle.community.devhub-dnfa.testnet
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/double-nested-factory-with-out-path-v1.0.0
const DOUBLE_NESTED_FACTORY_PRODUCT_WITH_OUT_PATH: TestCase = TestCase {
    input: r#"{
  "version": "0.1.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/12d1b6558444b2aab00aee3a03cdb3aa729d5006",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "contract_path": "discussions",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=12d1b6558444b2aab00aee3a03cdb3aa729d5006",
    "output_wasm_path": "/home/near/code/target/near/devhub_discussions/devhub_discussions.wasm"
  }
}"#,
    expected_output: "DYqoSEdkdzyfXF22F4YrKV6o9vBC2vx9NKXCdZMqvWRU",
};

#[test]
fn test_double_nested_factory_product_with_out_path() -> eyre::Result<()> {
    common_verify_test_routine(DOUBLE_NESTED_FACTORY_PRODUCT_WITH_OUT_PATH)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/uniquehandle.community.devhub-dnfa.testnet
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/double-nested-factory-with-out-path-v1.0.0
const DOUBLE_NESTED_FACTORY_2ND_LEVEL_WITH_OUT_PATH: TestCase = TestCase {
    input: r#"{
  "version": "0.1.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/12d1b6558444b2aab00aee3a03cdb3aa729d5006",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "contract_path": "community",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=12d1b6558444b2aab00aee3a03cdb3aa729d5006",
    "output_wasm_path": "/home/near/code/target/near/devhub_community/devhub_community.wasm"
  }
}"#,
    expected_output: "8n3ukXqv2TyBbvhrSus1tmXxkFNGLmcNgC76Q3rRxsSk",
};

#[test]
fn test_double_nested_factory_2nd_level_with_out_path() -> eyre::Result<()> {
    common_verify_test_routine(DOUBLE_NESTED_FACTORY_2ND_LEVEL_WITH_OUT_PATH)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/community.devhub-dnfa.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/double-nested-factory-with-out-path-v1.0.0
const DOUBLE_NESTED_FACTORY_1ST_LEVEL_WITH_OUT_PATH: TestCase = TestCase {
    input: r#"{
  "version": "0.1.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/12d1b6558444b2aab00aee3a03cdb3aa729d5006",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "contract_path": "community-factory",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=12d1b6558444b2aab00aee3a03cdb3aa729d5006",
    "output_wasm_path": "/home/near/code/target/near/devhub_community_factory/devhub_community_factory.wasm"
  }
}"#,
    expected_output: "HUT9ogFQUfybpESvq4vCrjgiXgBCV2ABGo6TqFx1nU4L",
};

#[test]
fn test_double_nested_factory_1st_level_with_out_path() -> eyre::Result<()> {
    common_verify_test_routine(DOUBLE_NESTED_FACTORY_1ST_LEVEL_WITH_OUT_PATH)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/factory-with-out-path-passed-env-a.testnet?tab=contract
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/factory-with-out-path-passed-env-v1.0.0
const SIMPLE_FACTORY_WITH_OUT_PATH_WITH_PASSED_ENV: TestCase = TestCase {
    input: r#"{
  "version": "1.0.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/119cc225e72e4a7889a87c8ea8d78f5c8ae349ee",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
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
    "contract_path": "workspace_root_folder/factory",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=119cc225e72e4a7889a87c8ea8d78f5c8ae349ee",
    "output_wasm_path": "/home/near/code/workspace_root_folder/target/near/simple_factory/simple_factory.wasm"
  }
}"#,
    expected_output: "8og8uknRTz1UxLrwjoDSNKAW6LhVTRFs4iDCRr45a6H1",
};

#[test]
fn test_simple_factory_with_out_path_with_passed_env() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_WITH_OUT_PATH_WITH_PASSED_ENV)?;
    Ok(())
}

/// https://testnet.nearblocks.io/address/product.factory-with-out-path-passed-env-a.testnet
/// https://github.com/dj8yfo/verify_contracts_collection/releases/tag/factory-with-out-path-passed-env-v1.0.0
const SIMPLE_FACTORY_PRODUCT_WITH_OUT_PATH_WITH_PASSED_ENV: TestCase = TestCase {
    input: r#"{
  "version": "1.1.0",
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/119cc225e72e4a7889a87c8ea8d78f5c8ae349ee",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.3.0"
    }
  ],
  "build_info": {
    "build_environment": "sourcescan/cargo-near:0.14.1-rust-1.86.0@sha256:eaac91be3119cc7c136b6f375f2d3e092001f717ed6151ccc9d5348c2d6a640c",
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
    "contract_path": "workspace_root_folder/product-donation",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=119cc225e72e4a7889a87c8ea8d78f5c8ae349ee",
    "output_wasm_path": "/home/near/code/workspace_root_folder/target/near/simple_factory_product/simple_factory_product.wasm"
  }
}"#,
    expected_output: "5Q8z2kJCTbfL7DEiZtsWa2UQNSizdQhpUY3d5krS6ziS",
};

#[test]
fn test_simple_factory_product_with_out_path_with_passed_env() -> eyre::Result<()> {
    common_verify_test_routine(SIMPLE_FACTORY_PRODUCT_WITH_OUT_PATH_WITH_PASSED_ENV)?;
    Ok(())
}

mod whitelist {

    use near_verify_rs::types::{
        contract_source_metadata::ContractSourceMetadata, whitelist::Whitelist,
    };

    use crate::TestCase;

    /// this test case is not checked out or compiled, only metadata validated
    const CONTRACT_WITH_NONSTANDARD_IMAGE: TestCase = TestCase {
        input: r#"{
  "build_info": {
    "build_command": [
      "cargo",
      "near",
      "build",
      "non-reproducible-wasm",
      "--locked"
    ],
    "build_environment": "dj8yfo/sourcescan:0.x.x-dev-pr-262@sha256:a231d4bf975d561a06dd5357f2ac03c883e8b3b510994f3b40c9b975dcdb02ce",
    "contract_path": "",
    "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=cb100096d0eb67654857949e1ff49fff2f385012"
  },
  "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/cb100096d0eb67654857949e1ff49fff2f385012",
  "standards": [
    {
      "standard": "nep330",
      "version": "1.2.0"
    }
  ],
  "version": "1.0.0"
}"#,
        expected_output: "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN",
    };

    #[test]
    fn test_simple_package_with_nonstandard_image() -> eyre::Result<()> {
        let whitelist: Whitelist = {
            let file = std::fs::read("tests/resources/whitelist_ok_nonstandard_image.json")
                .expect("no std:fs::read error");
            serde_json::from_slice(&file).expect("no serde_json::from_slice error")
        };
        let contract_source_metadata: ContractSourceMetadata =
            serde_json::from_str(CONTRACT_WITH_NONSTANDARD_IMAGE.input)?;

        contract_source_metadata.validate(Some(whitelist))?;
        Ok(())
    }

    mod decline {
        use near_verify_rs::types::{
            contract_source_metadata::ContractSourceMetadata, whitelist::Whitelist,
        };

        use crate::{whitelist::CONTRACT_WITH_NONSTANDARD_IMAGE, TestCase};

        #[test]
        fn test_decline_simple_package_with_unexpected_image() -> eyre::Result<()> {
            let whitelist: Whitelist = {
                let file = std::fs::read("tests/resources/whitelist_err_image.json")
                    .expect("no std:fs::read error");
                serde_json::from_slice(&file).expect("no serde_json::from_slice error")
            };

            let contract_source_metadata: ContractSourceMetadata =
                serde_json::from_str(CONTRACT_WITH_NONSTANDARD_IMAGE.input)?;

            let Err(err) = contract_source_metadata.validate(Some(whitelist)) else {
                panic!("Expecting an error returned from `contract_source_metadata.validate`");
            };
            println!("{:#?}", err);

            assert!(format!("{:?}", err).contains("no matching entry found for"));
            Ok(())
        }

        /// this test case is not checked out or compiled, only metadata validated
        const SIMPLE_PACKAGE_WITH_INVALID_OUT_PATH: TestCase = TestCase {
            input: r#"{
                "build_info": {
                  "build_command": [
                    "cargo",
                    "near",
                    "build",
                    "non-reproducible-wasm",
                    "--locked"
                  ],
                  "build_environment": "dj8yfo/sourcescan:0.14.0-rust-1.85.1@sha256:2dacaf4582374a02ed6a88fc1b285d418cd8b055d7436415bff87b6dfca0f167",
                  "contract_path": "",
                  "output_wasm_path": "/home/bear/code/target/near/simple_package_with_output_path.wasm",
                  "source_code_snapshot": "git+https://github.com/dj8yfo/verify_contracts_collection?rev=18747ed2d0108c767d282cd71fadc126735f3840"
                },
                "link": "https://github.com/dj8yfo/verify_contracts_collection/tree/18747ed2d0108c767d282cd71fadc126735f3840",
                "standards": [
                  {
                    "standard": "nep330",
                    "version": "1.3.0"
                  }
                ],
                "version": "1.0.0"
              }"#,
            expected_output: "3BxUrFTmaz2WKtzMTtH9MbPATW8ME4RjMbXiR2pfb1q5",
        };
        #[test]
        fn test_decline_simple_package_with_invalid_out_path() -> eyre::Result<()> {
            let contract_source_metadata: ContractSourceMetadata =
                serde_json::from_str(SIMPLE_PACKAGE_WITH_INVALID_OUT_PATH.input)?;

            let Err(err) = contract_source_metadata.validate(None) else {
                panic!("Expecting an error returned from `contract_source_metadata.validate`");
            };
            println!("{:#?}", err);

            assert!(format!("{:?}", err).contains("isn't a subpath of `/home/near/code`"));
            Ok(())
        }
    }
}
