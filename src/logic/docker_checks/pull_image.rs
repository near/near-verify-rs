use std::process::Stdio;

use colored::Colorize;

use crate::pretty_print::quiet_println;

pub fn check(docker_image: &str, quiet: bool) -> eyre::Result<()> {
    quiet_println!(
        quiet,
        "{} {}",
        "docker image to be used:".green(),
        docker_image
    );
    quiet_println!(quiet,);

    let mut docker_cmd = docker_pull_cmd(docker_image);

    let err_report = format!("Image `{}` could not be found in registry!", docker_image);
    if quiet {
        docker_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    }
    let status_result = docker_cmd.status();
    let status = super::handle_io_error(
        &docker_cmd,
        status_result,
        eyre::eyre!(err_report.clone()),
        quiet,
    )?;
    if !status.success() {
        super::print::command_status(status, docker_cmd, quiet);
        return Err(eyre::eyre!(err_report));
    }
    Ok(())
}

fn docker_pull_cmd(image: &str) -> std::process::Command {
    let docker_cmd: std::process::Command = {
        let docker_args = {
            let mut docker_args = vec!["pull"];
            docker_args.push(image);
            docker_args
        };

        let mut docker_cmd = std::process::Command::new("docker");
        docker_cmd.arg("image");
        docker_cmd.args(docker_args);
        docker_cmd
    };
    docker_cmd
}
