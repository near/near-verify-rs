//! Docker integration tests for the boundary between online fetching and offline execution.
#![cfg(unix)]

use camino::Utf8PathBuf;
use near_verify_rs::{
    logic::nep330_build,
    types::contract_source_metadata::{BuildInfo, ContractSourceMetadata},
};
use std::{fs, net::TcpListener, os::unix::fs::PermissionsExt};

// Stable, digest-pinned image already used by the existing contract-hash fixtures.
const IMAGE: &str = "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2";

struct Fixture {
    _directory: tempfile::TempDir,
    path: Utf8PathBuf,
    metadata: ContractSourceMetadata,
}

impl Fixture {
    fn new(build_script: &str) -> eyre::Result<Self> {
        let directory = tempfile::tempdir()?;
        let path = Utf8PathBuf::from_path_buf(directory.path().to_path_buf()).unwrap();
        fs::create_dir(path.join("src"))?;
        for (file, contents) in [
            (
                "Cargo.toml",
                "[package]\nname = \"offline_fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n",
            ),
            (
                "Cargo.lock",
                "version = 4\n\n[[package]]\nname = \"offline_fixture\"\nversion = \"0.1.0\"\n",
            ),
            (
                "src/lib.rs",
                "#[no_mangle]\npub extern \"C\" fn answer() -> u32 { 42 }\n",
            ),
            ("build.rs", build_script),
        ] {
            fs::write(path.join(file), contents)?;
        }
        let metadata = ContractSourceMetadata {
            build_info: Some(BuildInfo {
                build_environment: IMAGE.to_string(),
                build_command: "cargo build --locked --release --target wasm32-unknown-unknown"
                    .split(' ')
                    .map(String::from)
                    .collect(),
                output_wasm_path: Some(
                    "/home/near/code/target/wasm32-unknown-unknown/release/offline_fixture.wasm"
                        .to_string(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        };
        metadata.validate(None)?;
        Ok(Self {
            _directory: directory,
            path,
            metadata,
        })
    }

    fn run_offline(&self, docker_args: Vec<String>) -> eyre::Result<Utf8PathBuf> {
        nep330_build::run_offline(self.metadata.clone(), self.path.clone(), docker_args, false)
    }
}

#[test]
fn offline_build_blocks_build_script_network() -> eyre::Result<()> {
    // The online control proves this endpoint is reachable from Docker. No public
    // Internet endpoint is needed, and a closed port cannot make the test pass.
    let listener = TcpListener::bind("0.0.0.0:0")?;
    listener.set_nonblocking(true)?;
    let port = listener.local_addr()?.port();
    let fixture = Fixture::new(&format!(
        r#"
use std::{{env, net::{{TcpStream, ToSocketAddrs}}, time::Duration}};
fn main() {{
    let address = "host.docker.internal:{port}".to_socket_addrs().unwrap().next().unwrap();
    let reachable = TcpStream::connect_timeout(&address, Duration::from_secs(5)).is_ok();
    let offline = env::var("CARGO_NET_OFFLINE").as_deref() == Ok("true");
    assert_eq!(reachable, !offline, "build script network isolation changed");
}}
"#
    ))?;
    // Docker Desktop provides this hostname itself. Linux needs an explicit alias.
    let docker_args: Vec<_> = cfg!(target_os = "linux")
        .then(|| "--add-host=host.docker.internal:host-gateway".to_string())
        .into_iter()
        .collect();
    let online = nep330_build::run(
        fixture.metadata.clone(),
        fixture.path.clone(),
        docker_args.clone(),
        false,
    )?;
    let online_wasm = fs::read(online)?;
    listener.accept()?;
    // Force build.rs to execute again rather than accepting Cargo's cached output.
    fs::remove_dir_all(fixture.path.join("target"))?;
    assert_eq!(online_wasm, fs::read(fixture.run_offline(docker_args)?)?);
    Ok(())
}

#[test]
fn offline_fetch_does_not_execute_repository_wrapper() -> eyre::Result<()> {
    let fixture = Fixture::new("fn main() {}\n")?;
    fs::create_dir(fixture.path.join(".cargo"))?;
    fs::write(
        fixture.path.join(".cargo/config.toml"),
        "[build]\nrustc-wrapper = \"/home/near/code/wrapper.sh\"\n",
    )?;
    let wrapper = fixture.path.join("wrapper.sh");
    fs::write(
        &wrapper,
        r#"#!/bin/sh
set -eu
if [ "${CARGO_NET_OFFLINE:-}" != true ]; then
    touch /home/near/code/wrapper-ran-online
    exit 97
fi
touch /home/near/code/wrapper-ran-offline
exec "$@"
"#,
    )?;
    fs::set_permissions(wrapper, fs::Permissions::from_mode(0o755))?;
    fixture.run_offline(vec![])?;
    assert!(!fixture.path.join("wrapper-ran-online").exists());
    assert!(fixture.path.join("wrapper-ran-offline").exists());
    Ok(())
}

#[test]
fn offline_fetch_failure_does_not_start_build() -> eyre::Result<()> {
    let mut fixture = Fixture::new("fn main() {}\n")?;
    // A missing lockfile must fail cargo fetch --locked before the build command.
    fs::remove_file(fixture.path.join("Cargo.lock"))?;
    fixture.metadata.build_info.as_mut().unwrap().build_command = [
        "/bin/sh",
        "-c",
        "touch /home/near/code/build-started; exit 99",
    ]
    .map(String::from)
    .to_vec();
    let error = fixture.run_offline(vec![]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "`cargo fetch` in docker container failed"
    );
    assert!(!fixture.path.join("build-started").exists());
    Ok(())
}
