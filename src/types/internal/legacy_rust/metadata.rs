use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{MetadataCommand, Package};
use eyre::{ContextCompat, WrapErr};

use crate::pretty_print;

use super::manifest_path::ManifestPath;
pub const EXPECTED_EXTENSION: &str = "wasm";

/// Relevant metadata obtained from Cargo.toml.
#[derive(Debug)]
pub struct CrateMetadata {
    pub root_package: Package,
    pub target_directory: Utf8PathBuf,
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

        let filename = format!("{}.{}", self.formatted_package_name(), EXPECTED_EXTENSION);

        Ok(output_dir.join(filename))
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
    // Artifact discovery needs workspace/package fields, not dependency resolution.
    // In particular, the host need not have the container's dependency cache.
    cmd.no_deps();
    let mut options = vec!["--offline".to_string()];
    if !no_locked {
        options.push("--locked".to_string());
    }
    cmd.other_options(options);
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
        .packages
        .iter()
        // --no-deps omits the resolve graph. root_package() then identifies the
        // workspace root, which is not necessarily the requested contract crate.
        .find(|package| {
            package
                .manifest_path
                .canonicalize_utf8()
                .is_ok_and(|path| path == manifest_path.path)
        })
        .wrap_err(
            "Cargo metadata did not contain the requested contract manifest.\n\
            Pass the contract crate's Cargo.toml rather than a virtual workspace manifest.",
        )?
        .clone();
    Ok((metadata, root_package))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_output_resolution_does_not_resolve_dependencies() -> eyre::Result<()> {
        for in_workspace in [false, true] {
            let directory = tempfile::tempdir()?;
            let root = Utf8PathBuf::from_path_buf(directory.path().to_path_buf()).unwrap();
            let contract = if in_workspace {
                std::fs::write(
                    root.join("Cargo.toml"),
                    "[workspace]\nmembers = [\"contract\"]\nresolver = \"2\"\n",
                )?;
                root.join("contract")
            } else {
                root.clone()
            };
            std::fs::create_dir_all(contract.join("src"))?;
            std::fs::write(
                contract.join("src/lib.rs"),
                "pub fn answer() -> u32 { 42 }\n",
            )?;
            // Output discovery needs only the package name and workspace layout.
            // Requiring this unavailable dependency would make it fail on a host
            // whose dependencies were fetched only into the Docker build cache.
            let unavailable = url::Url::from_directory_path(root.join("unavailable-dependency"))
                .map_err(|()| eyre::eyre!("invalid fixture path"))?;
            std::fs::write(
                contract.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"legacy-contract\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nunavailable = {{ git = \"{unavailable}\" }}\n"
                ),
            )?;
            let manifest = ManifestPath::try_from(contract.join("Cargo.toml"))?;
            let metadata = CrateMetadata::collect(manifest, false)?;
            assert_eq!(metadata.root_package.name.as_str(), "legacy-contract");
            let output = metadata.get_legacy_cargo_near_output_path()?;
            let suffix = if in_workspace {
                "near/legacy_contract/legacy_contract.wasm"
            } else {
                "near/legacy_contract.wasm"
            };
            assert!(output.ends_with(suffix), "unexpected output path: {output}");
            assert!(!root.join("Cargo.lock").exists());
        }
        Ok(())
    }
}
