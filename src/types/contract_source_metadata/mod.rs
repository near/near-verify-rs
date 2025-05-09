pub use build_info::BuildInfo;
use serde::{Deserialize, Serialize};

use crate::env_keys;

mod validate;
/// The struct provides information about deployed contract's source code and supported standards.
///
/// Contract source metadata follows [**NEP-330 standard**](https://github.com/near/NEPs/blob/master/neps/nep-0330.md) for smart contracts
#[derive(Debug, Clone, PartialEq, Default, Eq, Serialize, Deserialize)]
pub struct ContractSourceMetadata {
    /// Optional version identifier, typically a semantic version
    ///
    /// ## Examples:
    ///
    /// ```rust,no_run
    /// # let version: Option<String> =
    /// // Semantic version
    /// Some("1.0.0".into())
    /// # ;
    /// ```
    /// ```rust,no_run
    /// # let version: Option<String> =
    /// // Git commit
    /// Some("39f2d2646f2f60e18ab53337501370dc02a5661c".into())
    /// # ;
    /// ```
    pub version: Option<String>,

    /// Optional URL to source code repository/tree
    ///
    /// ## Examples:
    ///
    /// ```rust,no_run
    /// # let link: Option<String> =
    /// // GitHub URL
    /// Some("https://github.com/org/repo/tree/8d8a8a0fe86a1d8eb3bce45f04ab1a65fecf5a1b".into())
    /// # ;
    /// ```
    /// ```rust,no_run
    /// # let link: Option<String> =
    /// // GitHub URL
    /// Some("https://github.com/near-examples/nft-tutorial".into())
    /// # ;
    /// ```
    /// ```rust,no_run
    /// # let link: Option<String> =
    /// // IPFS CID
    /// Some("bafybeiemxf5abjwjbikoz4mc3a3dla6ual3jsgpdr4cjr3oz3evfyavhwq".into())
    /// # ;
    /// ```
    pub link: Option<String>,

    /// List of supported NEAR standards (NEPs) with their versions
    ///
    /// This field is an addition of **1.1.0** **NEP-330** revision
    ///
    /// ## Examples:
    ///
    /// This field will always include NEP-330 itself:
    /// ```rust,no_run
    /// # use near_verify_rs::types::nep330::Standard;
    /// # let link: Vec<Standard> =
    /// // this is always at least 1.1.0
    /// vec![Standard { standard: "nep330".into(), version: "1.1.0".into() }]
    /// # ;
    /// ```
    /// ```rust,no_run
    /// # use near_verify_rs::types::nep330::Standard;
    /// # let link: Vec<Standard> =
    /// vec![Standard { standard: "nep330".into(), version: "1.2.0".into() }]
    /// # ;
    /// ```
    // it's a guess it was added as 1.1.0 of nep330, [nep330 1.1.0 standard recording](https://www.youtube.com/watch?v=pBLN9UyE6AA) actually discusses nep351
    #[serde(default)]
    pub standards: Vec<Standard>,

    /// Optional details that are required for formal contract WASM build reproducibility verification
    ///
    /// This field is an addition of **1.2.0** **NEP-330** revision
    pub build_info: Option<BuildInfo>,
}

impl ContractSourceMetadata {
    pub fn docker_env_args(&self) -> Vec<String> {
        let mut result = vec![];
        if let Some(ref build_info) = self.build_info {
            result.extend(vec![
                "--env".to_string(),
                format!(
                    "{}={}",
                    env_keys::BUILD_ENVIRONMENT,
                    build_info.build_environment
                ),
            ]);
            result.extend(vec![
                "--env".to_string(),
                format!(
                    "{}={}",
                    env_keys::SOURCE_CODE_SNAPSHOT,
                    build_info.source_code_snapshot
                ),
            ]);
            result.extend(vec![
                "--env".to_string(),
                format!("{}={}", env_keys::CONTRACT_PATH, build_info.contract_path),
            ]);
            if let Some(ref output_wasm_path) = build_info.output_wasm_path {
                result.extend(vec![
                    "--env".to_string(),
                    format!("{}={}", env_keys::OUTPUT_WASM_PATH, output_wasm_path),
                ]);
            }
        }

        if let Some(ref repo_link_hint) = self.link {
            result.extend(vec![
                "--env".to_string(),
                format!("{}={}", env_keys::LINK, repo_link_hint),
            ]);
        }

        result
    }
}
/// NEAR Standard implementation descriptor following [NEP-330](https://github.com/near/NEPs/blob/master/neps/nep-0330.md)    
#[derive(Debug, Clone, PartialEq, Default, Eq, Serialize, Deserialize)]
pub struct Standard {
    /// Standard name in lowercase NEP format
    ///
    /// ## Examples:
    ///
    /// ```rust,no_run
    /// # let standard: String =
    /// // for fungible tokens
    /// "nep141".into()
    /// # ;
    /// ```
    pub standard: String,

    /// Implemented standard version using semantic versioning
    ///
    /// ## Examples:
    ///
    /// ```rust,no_run
    /// # let version: String =
    /// // for initial release
    /// "1.0.0".into()
    /// # ;
    /// ```
    pub version: String,
}

mod build_info {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Default, Eq, Serialize, Deserialize)]
    /// Defines all required details for formal WASM build reproducibility verification
    /// according to [**NEP-330 standard 1.2.0 revision**](https://github.com/near/NEPs/blob/master/neps/nep-0330.md)
    pub struct BuildInfo {
        /// Reference to a reproducible build environment docker image
        ///
        /// ## Examples:
        ///
        /// ```rust,no_run
        /// # let build_environment: String =  
        ///  "sourcescan/cargo-near:0.13.3-rust-1.84.0@sha256:722198ddb92d1b82cbfcd3a4a9f7fba6fd8715f4d0b5fb236d8725c4883f97de".into()
        /// # ;
        /// ```
        pub build_environment: String,
        /// The exact command that was used to build the contract, with all the flags
        ///
        /// ## Examples:
        ///
        /// ```rust,no_run
        /// # let build_command: Vec<String> =
        /// vec![
        ///     "cargo".into(),
        ///     "near".into(),
        ///     "build".into(),
        ///     "non-reproducible-wasm".into(),
        ///     "--locked".into()
        /// ]
        /// # ;
        /// ```
        pub build_command: Vec<String>,
        /// Relative path to contract crate within the source code
        ///
        /// ## Examples:
        ///
        /// ```rust,no_run
        /// # let contract_path: String =
        /// "near/omni-prover/wormhole-omni-prover-proxy".into()
        /// # ;
        /// ```
        /// ```rust,no_run
        /// # let contract_path: String =
        /// // root of a repo
        /// "".into()
        /// # ;
        /// ```
        pub contract_path: String,
        /// Reference to the source code snapshot that was used to build the contract
        ///
        /// ## Examples:
        ///
        /// ```rust,no_run
        /// # let source_code_snapshot: String =
        /// "git+https://github.com/org/repo?rev=8d8a8a0fe86a1d8eb3bce45f04ab1a65fecf5a1b".into()
        /// # ;
        /// ```
        pub source_code_snapshot: String,
        /// A path within the build environment, where the result WASM binary has been put
        /// during build.
        /// This should be a subpath of `/home/near/code`
        ///
        /// This field is an addition of **1.3.0** **NEP-330** revision
        ///
        /// ## Examples:
        ///
        /// ```rust,no_run
        /// # let output_wasm_path: Option<String> =
        /// Some("/home/near/code/target/near/simple_package.wasm".into())
        /// # ;
        /// ```
        pub output_wasm_path: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use super::ContractSourceMetadata;

    const OLD_1_0_METADATA: &str = r#"{
  "link": "https://github.com/old/contract_repo",
  "version": "1.0.0"
}"#;

    #[test]
    fn test_parse_old_1_0_metadata() -> eyre::Result<()> {
        let contract_source_metadata: ContractSourceMetadata =
            serde_json::from_str(OLD_1_0_METADATA)?;

        assert_eq!(contract_source_metadata.version, Some("1.0.0".to_owned()));
        assert_eq!(
            contract_source_metadata.link,
            Some("https://github.com/old/contract_repo".to_owned())
        );
        Ok(())
    }
}
