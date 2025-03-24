pub fn shell_escape_nep330_build_command(build_command: Vec<String>) -> String {
    tracing::debug!("cli_build_command_in_docker {:#?}", build_command);
    shell_words::join(build_command)
}
