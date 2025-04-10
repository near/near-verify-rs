use colored::Colorize;

use crate::pretty_print::quiet_println;

const ERR_SANITY: &str = "`docker` sanity check failed!";

const PERM_DENIED_STATUS: i32 = 126;

pub fn check(quiet: bool) -> eyre::Result<()> {
    let mut docker_cmd = std::process::Command::new("docker");
    docker_cmd.args(["run", "--rm", "hello-world"]);
    let output_result = docker_cmd.output();
    let output =
        super::handle_io_error(&docker_cmd, output_result, eyre::eyre!(ERR_SANITY), quiet)?;

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr)?;
        quiet_println!(quiet,);
        quiet_println!(quiet, "{}", stderr.yellow());
        if permission_denied(&output.status, stderr)? {
            quiet_println!(quiet, "{}", "Permission denied!".cyan());
            super::print::installation_links(quiet);
            super::print::linux_postinstall_steps(quiet);
        } else {
            super::print::installation_links(quiet);
        }
        super::print::command_status(output.status, docker_cmd, quiet);
        return Err(eyre::eyre!(ERR_SANITY));
    }
    Ok(())
}

fn permission_denied(status: &std::process::ExitStatus, stderr: &str) -> eyre::Result<bool> {
    let exit_code_match = status.code().unwrap_or(-1) == PERM_DENIED_STATUS;
    let stderr_match = stderr.to_lowercase().contains("permission denied");
    Ok(exit_code_match || stderr_match)
}
