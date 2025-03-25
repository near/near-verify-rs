/// TODO #B: move to logic module 
pub mod docker_checks;
/// TODO #B: move to logic module 
pub mod docker_command;
pub mod types;

mod logic {
    pub fn shell_escape_nep330_build_command(build_command: Vec<String>) -> String {
        tracing::debug!("cli_build_command_in_docker {:#?}", build_command);
        shell_words::join(build_command)
    }
}



/// module contains variables, which are set to configure build with WASM reproducibility,
/// which correspond to some fields of `ContractSourceMetadata` in <https://github.com/near/NEPs/blob/master/neps/nep-0330.md>
pub mod env_keys;


pub use logic::shell_escape_nep330_build_command;