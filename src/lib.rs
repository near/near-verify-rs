pub mod types {
    pub mod contract_source_metadata;
    pub mod source_id;

    pub mod sha256_checksum;

    pub(crate) mod internal {
        pub mod container_paths;
        /// 1. this module is needed to compute legacy NEP330-1.2.0 rust crates' output paths (from docker container builds)
        /// 2. yet-to-be NEP330-1.3.0 `build_info.result_path` extension will make usage of these
        ///    modules redundant for newer contracts where these path `build_info.result_path`
        ///    will be set (will be [Option::Some])
        pub mod legacy_rust {
            pub mod manifest_path;
            pub mod metadata;

            pub use manifest_path::ManifestPath;
            pub use metadata::CrateMetadata;
        }
    }
}

pub mod logic {
    pub const NEP330_REPO_MOUNT: &str = "/home/near/code";

    pub fn shell_escape_nep330_build_command(build_command: Vec<String>) -> String {
        tracing::debug!("cli_build_command_in_docker {:#?}", build_command);
        shell_words::join(build_command)
    }
    pub mod nep330_build;
    pub mod docker_checks {
        use crate::logic::internal::docker_command::handle_io_error;
        use crate::logic::internal::docker_command::print;

        pub mod pull_image;
        pub mod sanity;
    }

    pub(crate) mod internal {
        pub mod docker_command;
    }
    pub fn compute_hash(
        path: camino::Utf8PathBuf,
    ) -> eyre::Result<crate::types::sha256_checksum::SHA256Checksum> {
        let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
        sha2::Digest::update(&mut hasher, std::fs::read(&path)?);
        let hash = sha2::Digest::finalize(hasher);
        let hash: &[u8] = hash.as_ref();
        Ok(crate::types::sha256_checksum::SHA256Checksum {
            hash: hash.to_vec(),
        })
    }
}

pub mod pretty_print {
    pub fn indent_payload(s: &str) -> String {
        use std::fmt::Write;

        let mut indented_string = String::new();
        indenter::indented(&mut indented_string)
            .with_str(" |    ")
            .write_str(s)
            .ok();
        indented_string
    }
}

/// module contains variables, which are set to configure build with WASM reproducibility,
/// which correspond to some fields of `ContractSourceMetadata` in <https://github.com/near/NEPs/blob/master/neps/nep-0330.md>
pub mod env_keys;
