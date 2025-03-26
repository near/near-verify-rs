use std::{thread, time::Duration};

use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{MetadataCommand, Package};
use colored::Colorize;
use eyre::{ContextCompat, OptionExt, WrapErr};

use crate::pretty_print;

use super::manifest_path::ManifestPath;

/// Relevant metadata obtained from Cargo.toml.
#[derive(Debug)]
pub struct CrateMetadata {
    pub root_package: Package,
    pub target_directory: Utf8PathBuf,
    pub manifest_path: ManifestPath,
    pub raw_metadata: cargo_metadata::Metadata,
}
/// Create the directory if it doesn't exist, and return the absolute path to it.
fn force_canonicalize_dir(dir: &Utf8Path) -> eyre::Result<Utf8PathBuf> {
    std::fs::create_dir_all(dir)
        .wrap_err_with(|| format!("failed to create directory `{}`", dir))?;
    // use canonicalize from `dunce` create instead of default one from std because it's compatible with Windows UNC paths
    // and don't break cargo compilation on Windows
    // https://github.com/rust-lang/rust/issues/42869
    Utf8PathBuf::from_path_buf(
        dunce::canonicalize(dir)
            .wrap_err_with(|| format!("failed to canonicalize path: {} ", dir))?,
    )
    .map_err(|err| eyre::eyre!("failed to convert path {}", err.to_string_lossy()))
}

impl CrateMetadata {
    /// Parses the contract manifest and returns relevant metadata.
    pub fn collect(manifest_path: ManifestPath, no_locked: bool) -> eyre::Result<Self> {
        let (metadata, root_package) = {
            let (mut metadata, root_package) = get_cargo_metadata(&manifest_path, no_locked)?;
            metadata.target_directory = force_canonicalize_dir(&metadata.target_directory)?;
            metadata.workspace_root = metadata.workspace_root.canonicalize_utf8()?;
            (metadata, root_package)
        };

        let mut target_directory = force_canonicalize_dir(&metadata.target_directory.join("near"))?;

        // Normalize the package and lib name.
        let package_name = root_package.name.replace('-', "_");

        let absolute_manifest_dir = manifest_path.directory()?;
        if absolute_manifest_dir != metadata.workspace_root {
            // If the contract is a package in a workspace, we use the package name
            // as the name of the sub-folder where we put the `.contract` bundle.
            target_directory = force_canonicalize_dir(&target_directory.join(package_name))?;
        }

        let crate_metadata = CrateMetadata {
            root_package,
            target_directory,
            manifest_path,
            raw_metadata: metadata,
        };
        tracing::trace!("crate metadata : {:#?}", crate_metadata);
        Ok(crate_metadata)
    }

    fn resolve_output_dir(&self) -> eyre::Result<Utf8PathBuf> {
        let result = self.target_directory.clone();
        tracing::info!(
            target: "near_teach_me",
            parent: &tracing::Span::none(),
            "Resolved output directory: {}", result
        );
        Ok(result)
    }
    fn formatted_package_name(&self) -> String {
        self.root_package.name.replace('-', "_")
    }

    pub fn get_legacy_cargo_near_output_path(&self) -> eyre::Result<camino::Utf8PathBuf> {
        let output_dir = self.resolve_output_dir()?;

        let filename = format!("{}.wasm", self.formatted_package_name());

        Ok(output_dir.join(filename.clone()))
    }
}
/// Get the result of `cargo metadata`, together with the root package id.
fn get_cargo_metadata(
    manifest_path: &ManifestPath,
    no_locked: bool,
) -> eyre::Result<(cargo_metadata::Metadata, Package)> {
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Fetching cargo metadata for {}", manifest_path.path
    );
    let mut cmd = MetadataCommand::new();
    if !no_locked {
        cmd.other_options(["--locked".to_string()]);
    }
    let cmd = cmd.manifest_path(&manifest_path.path);
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Command execution:\n{}",
        pretty_print::indent_payload(&format!("{:#?}", cmd.cargo_command()))
    );
    let metadata = cmd.exec();
    if let Err(cargo_metadata::Error::CargoMetadata { stderr }) = metadata.as_ref() {
        if stderr.contains("remove the --locked flag") {
            return Err(cargo_metadata::Error::CargoMetadata {
                stderr: stderr.clone(),
            })
            .wrap_err("Cargo.lock is absent or not up-to-date");
        }
    }
    let metadata = metadata
        .wrap_err("Error invoking `cargo metadata`. Your `Cargo.toml` file is likely malformed")?;
    let root_package = metadata
        .root_package()
        .wrap_err(
            "raw_metadata.root_package() returned None.\n\
            Command was likely called from a root of virtual workspace as current directory \
            and not from a contract's crate",
        )?
        .clone();
    Ok((metadata, root_package))
}
