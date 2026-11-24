use camino::Utf8PathBuf;
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
impl CrateMetadata {
    /// Parses the contract manifest and returns relevant metadata.
    ///
    /// The manifest belongs to the repository being verified, so this runs
    /// `cargo metadata --no-deps`: resolving dependencies would make the host
    /// fetch whatever sources that repository names. Nothing is created on disk;
    /// the build already produced the artifact if it succeeded.
    pub fn collect(manifest_path: ManifestPath) -> eyre::Result<Self> {
        let (metadata, root_package) = get_cargo_metadata(&manifest_path)?;
        let workspace_root = metadata.workspace_root.canonicalize_utf8()?;

        let mut target_directory = metadata.target_directory.join("near");

        let absolute_manifest_dir = manifest_path.directory()?;
        if absolute_manifest_dir != workspace_root {
            // If the contract is a package in a workspace, we use the package name
            // as the name of the sub-folder where we put the `.contract` bundle.
            target_directory = target_directory.join(root_package.name.replace('-', "_"));
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
) -> eyre::Result<(cargo_metadata::Metadata, Package)> {
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Fetching cargo metadata for {}", manifest_path.path
    );
    let mut cmd = MetadataCommand::new();
    // Skip dependency resolution: it would fetch the sources the repository names
    // (with the host's credentials), and the build in the container already used
    // the lockfile.
    cmd.no_deps();
    let cmd = cmd.manifest_path(&manifest_path.path);
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "Command execution:\n{}",
        pretty_print::indent_payload(&format!("{:#?}", cmd.cargo_command()))
    );
    let metadata = cmd
        .exec()
        .wrap_err("Error invoking `cargo metadata`. Your `Cargo.toml` file is likely malformed")?;
    let root_package = metadata
        .packages
        .iter()
        // Without a resolve graph, `root_package()` returns the workspace root package.
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
    use camino::Utf8Path;

    #[test]
    fn legacy_output_path_skips_dependency_resolution() -> eyre::Result<()> {
        for (in_workspace, expected) in [
            (false, "near/legacy_contract.wasm"),
            (true, "near/legacy_contract/legacy_contract.wasm"),
        ] {
            let directory = tempfile::tempdir()?;
            let root = Utf8Path::from_path(directory.path()).unwrap();
            let contract = if in_workspace {
                std::fs::write(
                    root.join("Cargo.toml"),
                    "[workspace]\nmembers = [\"contract\"]\nresolver = \"2\"\n",
                )?;
                root.join("contract")
            } else {
                root.to_owned()
            };
            std::fs::create_dir_all(contract.join("src"))?;
            std::fs::write(contract.join("src/lib.rs"), "")?;
            // Unavailable on the host, like dependencies fetched only into the Docker cache.
            let unavailable = url::Url::from_directory_path(root.join("unavailable")).unwrap();
            std::fs::write(
                contract.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"legacy-contract\"\nedition = \"2021\"\n\n[dependencies]\nunavailable = {{ git = \"{unavailable}\" }}\n"
                ),
            )?;
            let manifest = ManifestPath::try_from(contract.join("Cargo.toml"))?;
            let output = CrateMetadata::collect(manifest)?.get_legacy_cargo_near_output_path()?;
            assert!(
                output.ends_with(expected),
                "unexpected output path: {output}"
            );
            assert!(!root.join("Cargo.lock").exists());
        }
        Ok(())
    }
}
