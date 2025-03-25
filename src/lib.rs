/// TODO #B: move to logic module
pub mod docker_checks;
/// TODO #B: move to logic module
pub mod docker_command;
pub mod types {
    pub mod contract_source_metadata;
    pub mod source_id;

    /// TODO #A: make pub(crate) visibility
    pub mod internal;
}

pub mod logic {
    pub const NEP330_REPO_MOUNT: &str = "/home/near/code";

    pub fn shell_escape_nep330_build_command(build_command: Vec<String>) -> String {
        tracing::debug!("cli_build_command_in_docker {:#?}", build_command);
        shell_words::join(build_command)
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
