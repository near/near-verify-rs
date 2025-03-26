use crate::logic::internal::docker_command;
use crate::types::internal::container_paths;
use colored::Colorize;
use std::io::IsTerminal;
use std::str::FromStr;
use std::{
    process::{Command, ExitStatus},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "linux")]
use nix::unistd::{getgid, getuid};

use crate::env_keys;
use crate::pretty_print;
use crate::types::contract_source_metadata::ContractSourceMetadata;

pub const ERR_REPRODUCIBLE: &str = "Reproducible build in docker container failed.";
mod output;

/// TODO #H4: add validation of [BuildInfoMixed::build_environment] with `images_whitelist` [Vec<String>] argument
/// TODO #H3: check [BuildInfoMixed::build_environment] for regex match
/// TODO #H2: add validation for `build_command`, that the vec isn't empty, and all tokens aren't empty
/// TODO #C: move this to a method on [ContractSourceMetadata]
/// TODO #C1: extract a [ContractSourceMetadata::validate_contract_path] method
fn validate_meta(contract_source_metadata: &ContractSourceMetadata) -> eyre::Result<()> {
    if contract_source_metadata.build_info.is_none() {
        return Err(eyre::eyre!(
            "`build_info` field of `ContractSourceMetadata` cannot be null"
        ));
    }

    let build_info = contract_source_metadata.build_info.as_ref().unwrap();
    match unix_path::PathBuf::from_str(&build_info.contract_path) {
        Err(err) => {
            return Err(eyre::eyre!(
                "`contract_path` field (`{}`) of `BuildInfo` isn't a valid unix path: {:#?}",
                build_info.contract_path,
                err,
            ));
        }
        Ok(path) => {
            if !path.is_relative() {
                return Err(eyre::eyre!(
                    "`contract_path` field (`{}`) of `BuildInfo` isn't a relative unix path",
                    build_info.contract_path,
                ));
            }
            for component in path.components() {
                let unix_str = component.as_unix_str();
                if let Err(err) = unix_str.to_owned().into_string() {
                    // this is somewhat impossible to reach, as the whole path was parsed from a [String]
                    return Err(eyre::eyre!(
                        "`contract_path` field (`{}`) of `BuildInfo` contains a component which is not a valid utf8 string: `{:?}",
                        build_info.contract_path,
                        err,
                    ));
                }
            }
        }
    }
    Ok(())
}

fn handle_docker_run_status(
    contract_source_metadata: ContractSourceMetadata,
    contract_source_workdir: camino::Utf8PathBuf,
    status: ExitStatus,
    command: Command,
) -> eyre::Result<camino::Utf8PathBuf> {
    if status.success() {
        // let build_info = contract_source_metadata.build_info.as_ref().expect(
        //     "cannot be [Option::None] as per [ContractSourceMetadata::validate_meta] check"
        // );
        // if build_info.wasm_result_path.is_none() branch ============
        output::rust_legacy_wasm_output_path(contract_source_metadata, contract_source_workdir)
        // ============

        // if build_info.wasm_result_path.is_some() branch ============
        // unimplemented!();
        // this is pending nep330 1.3.0 extension
        // ============
    } else {
        docker_command::print::command_status(status, command);
        Err(eyre::eyre!(ERR_REPRODUCIBLE))
    }
}

pub fn run(
    contract_source_metadata: ContractSourceMetadata,
    contract_source_workdir: camino::Utf8PathBuf,
    additional_docker_args: Vec<String>,
) -> eyre::Result<camino::Utf8PathBuf> {
    let (status, command) = run_inner(
        contract_source_metadata.clone(),
        contract_source_workdir.clone(),
        additional_docker_args,
    )?;

    handle_docker_run_status(
        contract_source_metadata,
        contract_source_workdir,
        status,
        command,
    )
}

fn run_inner(
    contract_source_metadata: ContractSourceMetadata,
    contract_source_workdir: camino::Utf8PathBuf,
    additional_docker_args: Vec<String>,
) -> eyre::Result<(ExitStatus, Command)> {
    validate_meta(&contract_source_metadata)?;
    let build_info = contract_source_metadata
        .build_info
        .clone()
        .expect("cannot be [Option::None] as per `validate_meta` check");
    let mut docker_cmd: Command = {
        // Platform-specific UID/GID retrieval

        // reason for this mapping is that on Linux the volume is mounted natively,
        // and thus the unprivileged user inside Docker container should be able to write
        // to the mounted folder that has the host user permissions,
        // not specifying this mapping results in UID=Docker-User owned files created in host system
        #[cfg(target_os = "linux")]
        let uid_gid = format!("{}:{}", getuid(), getgid());
        #[cfg(not(target_os = "linux"))]
        let uid_gid = "1000:1000".to_string();

        let docker_container_name = {
            // Cross-platform process ID and timestamp
            let pid = std::process::id().to_string();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string();
            format!("near-verify-rs-{}-{}", timestamp, pid)
        };
        let container_paths =
            container_paths::Paths::compute(&build_info, contract_source_workdir)?;

        let docker_env_args = contract_source_metadata.docker_env_args();
        let shell_escaped_cargo_cmd =
            crate::logic::shell_escape_nep330_build_command(build_info.build_command);
        println!(
            "{} {}",
            "build command in container:".green(),
            shell_escaped_cargo_cmd
        );
        println!();

        let docker_args = {
            let mut docker_args = vec![
                "-u",
                &uid_gid,
                "--name",
                &docker_container_name,
                "--volume",
                &container_paths.host_volume_arg,
                "--rm",
                "--workdir",
                &container_paths.crate_path,
            ];
            let stdin_is_terminal = std::io::stdin().is_terminal();
            tracing::debug!("input device is a tty: {}", stdin_is_terminal);
            if stdin_is_terminal
                && std::env::var(env_keys::nonspec::SERVER_DISABLE_INTERACTIVE).is_err()
            {
                docker_args.push("-it");
            }

            docker_args.extend(docker_env_args.iter().map(|string| string.as_str()));
            docker_args.extend(additional_docker_args.iter().map(|string| string.as_str()));
            docker_args.extend(vec![&build_info.build_environment, "/bin/bash", "-c"]);

            docker_args.push(&shell_escaped_cargo_cmd);
            docker_args
        };

        let mut docker_cmd = Command::new("docker");
        docker_cmd.arg("run");
        docker_cmd.args(docker_args);
        docker_cmd
    };
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Docker command:\n{}",
        pretty_print::indent_payload(&format!("{:#?}", docker_cmd))
    );

    let status_result = docker_cmd.status();
    let status =
        docker_command::handle_io_error(&docker_cmd, status_result, eyre::eyre!(ERR_REPRODUCIBLE))?;

    Ok((status, docker_cmd))
}
