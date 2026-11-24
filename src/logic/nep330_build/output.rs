pub mod rust_legacy {
    use std::str::FromStr;

    use eyre::{Context, ContextCompat};

    use crate::{
        logic::nep330_build::output::common,
        types::{
            contract_source_metadata::ContractSourceMetadata,
            internal::legacy_rust::{
                CrateMetadata, ManifestPath, manifest_path::MANIFEST_FILE_NAME,
            },
        },
    };

    fn manifest_path(
        contract_source_metadata: ContractSourceMetadata,
        contract_source_workdir: camino::Utf8PathBuf,
    ) -> eyre::Result<camino::Utf8PathBuf> {
        let contract_path = {
            let contract_path = contract_source_metadata
                .build_info
                .wrap_err(
                    "cannot be [Option::None] as per [ContractSourceMetadata::validate_meta] check",
                )?
                .contract_path;
            unix_path::PathBuf::from_str(&contract_path).wrap_err(
                "should be a valid relative [unix_path::PathBuf] as per [ContractSourceMetadata::validate_meta] check",
            )?
        };
        let components = {
            let iterator = contract_path.components().map(|component| {
                let unix_str = component.as_unix_str();
                let string = unix_str
                    .to_owned()
                    .into_string()
                    .map_err(
                        |err| eyre::eyre!(
                            "should be a valid utf8 [String] component as per [ContractSourceMetadata::validate_meta] check: {:?}", 
                            err
                        )
                    )?;
                Ok(camino::Utf8PathBuf::from(string))
            });
            eyre::Result::<Vec<camino::Utf8PathBuf>>::from_iter(iterator)
        }?;
        let mut path = contract_source_workdir.clone();
        path.extend(components);
        path.push(MANIFEST_FILE_NAME);
        Ok(path)
    }

    pub fn wasm_output_path(
        contract_source_metadata: ContractSourceMetadata,
        contract_source_workdir: camino::Utf8PathBuf,
    ) -> eyre::Result<camino::Utf8PathBuf> {
        let checkout = common::canonical_checkout(&contract_source_workdir)?;
        let manifest_path = {
            let manifest_path = manifest_path(contract_source_metadata, contract_source_workdir)?;
            ManifestPath::try_from(manifest_path).wrap_err("Assumption about compiling a rust crate in docker container is invalid: manifest file not found")?
        };
        // Checked before `cargo metadata` reads it: a symlinked `Cargo.toml` must not
        // make the host parse a manifest outside the checkout.
        common::ensure_inside_checkout(&manifest_path.path, &checkout, "manifest")?;

        let crate_metadata = CrateMetadata::collect(manifest_path)?;

        let path = crate_metadata.get_legacy_cargo_near_output_path()?;
        tracing::info!(
            target: "near_teach_me",
            parent: &tracing::Span::none(),
            "assumed artifact result path for a rust crate docker build: `{}`", path
        );

        common::path_sane_check(&path, &checkout, "rust crate")
    }
}

pub mod explicit_metadata {
    use eyre::ContextCompat;

    use crate::logic::NEP330_REPO_MOUNT;

    use super::common;

    pub fn wasm_output_path(
        output_wasm_path: &str,
        contract_source_workdir: camino::Utf8PathBuf,
    ) -> eyre::Result<camino::Utf8PathBuf> {
        let base = camino::Utf8PathBuf::from(NEP330_REPO_MOUNT);
        let subpath = camino::Utf8PathBuf::from(output_wasm_path);

        let relative_path = pathdiff::diff_utf8_paths(&subpath, &base).wrap_err(format!(
            "cannot compute contract output pathdiff from mount point {}",
            NEP330_REPO_MOUNT
        ))?;
        let checkout = common::canonical_checkout(&contract_source_workdir)?;
        let wasm_path = contract_source_workdir.join(relative_path);
        common::path_sane_check(&wasm_path, &checkout, "generic nep330 1.3.0 compliant")
    }
}

mod common {
    use eyre::WrapErr;

    use crate::types::internal::legacy_rust::metadata::EXPECTED_EXTENSION;

    pub(super) fn canonical_checkout(
        contract_source_workdir: &camino::Utf8Path,
    ) -> eyre::Result<camino::Utf8PathBuf> {
        contract_source_workdir
            .canonicalize_utf8()
            .wrap_err_with(|| {
                format!("failed to resolve source checkout `{contract_source_workdir}`")
            })
    }

    /// Fails unless `canonical_path` lies inside the canonical `checkout`.
    ///
    /// The paths the host reads come from contract metadata and from files in the
    /// checkout, which the repository and the build container control. Neither `..`
    /// nor symlinks may lead the host outside the checkout.
    pub(super) fn ensure_inside_checkout(
        canonical_path: &camino::Utf8Path,
        checkout: &camino::Utf8Path,
        descriptor: &str,
    ) -> eyre::Result<()> {
        if !canonical_path.starts_with(checkout) {
            return Err(eyre::eyre!(
                "{descriptor} path `{canonical_path}` resolves outside the source checkout `{checkout}`"
            ));
        }
        Ok(())
    }

    /// Checks the artifact path and returns it resolved.
    pub(super) fn path_sane_check(
        path: &camino::Utf8PathBuf,
        checkout: &camino::Utf8Path,
        descriptor: &str,
    ) -> eyre::Result<camino::Utf8PathBuf> {
        if !path.exists() {
            return Err(eyre::eyre!(
                "assumed artifact result path for a {} docker build not found: `{}`",
                descriptor,
                path
            ));
        }
        let resolved = path
            .canonicalize_utf8()
            .wrap_err_with(|| format!("failed to resolve artifact path `{path}`"))?;
        ensure_inside_checkout(&resolved, checkout, "artifact")?;
        if !resolved.is_file() {
            return Err(eyre::eyre!("result path isn't a file: `{}`", path));
        }
        if resolved.extension() != Some(EXPECTED_EXTENSION) {
            return Err(eyre::eyre!(
                "result path doesn't have a `wasm` extension: `{}`",
                path
            ));
        }
        Ok(resolved)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use crate::types::contract_source_metadata::{BuildInfo, ContractSourceMetadata};

    /// A checkout and a directory next to it holding a file the host must not read.
    fn checkout_with_outside_secret()
    -> (tempfile::TempDir, camino::Utf8PathBuf, camino::Utf8PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let base = camino::Utf8Path::from_path(root.path()).unwrap().to_owned();
        let checkout = base.join("checkout");
        std::fs::create_dir_all(&checkout).unwrap();
        std::fs::create_dir_all(base.join("outside")).unwrap();
        std::fs::write(base.join("outside/secret.wasm"), b"secret").unwrap();
        std::fs::write(
            base.join("outside/Cargo.toml"),
            "[package]\nname = \"secret\"\n",
        )
        .unwrap();
        (root, base, checkout)
    }

    #[test]
    fn explicit_output_path_rejects_parent_components() {
        let (_root, _base, checkout) = checkout_with_outside_secret();
        let err = super::explicit_metadata::wasm_output_path(
            "/home/near/code/../outside/secret.wasm",
            checkout,
        )
        .unwrap_err();
        assert!(
            format!("{err:?}").contains("resolves outside the source checkout"),
            "{err:?}"
        );
    }

    #[test]
    fn explicit_output_path_rejects_symlink_out_of_checkout() {
        let (_root, base, checkout) = checkout_with_outside_secret();
        std::os::unix::fs::symlink(base.join("outside/secret.wasm"), checkout.join("out.wasm"))
            .unwrap();
        let err = super::explicit_metadata::wasm_output_path("/home/near/code/out.wasm", checkout)
            .unwrap_err();
        assert!(
            format!("{err:?}").contains("resolves outside the source checkout"),
            "{err:?}"
        );
    }

    #[test]
    fn explicit_output_path_inside_checkout_is_resolved() {
        let (_root, _base, checkout) = checkout_with_outside_secret();
        std::fs::create_dir_all(checkout.join("target")).unwrap();
        std::fs::write(checkout.join("target/contract.wasm"), b"wasm").unwrap();
        let path = super::explicit_metadata::wasm_output_path(
            "/home/near/code/target/contract.wasm",
            checkout.clone(),
        )
        .unwrap();
        assert_eq!(
            path,
            checkout
                .canonicalize_utf8()
                .unwrap()
                .join("target/contract.wasm")
        );
    }

    #[test]
    fn legacy_rejects_symlinked_manifest_out_of_checkout() {
        let (_root, base, checkout) = checkout_with_outside_secret();
        std::os::unix::fs::symlink(base.join("outside/Cargo.toml"), checkout.join("Cargo.toml"))
            .unwrap();
        let metadata = ContractSourceMetadata {
            build_info: Some(BuildInfo::default()),
            ..Default::default()
        };
        let err = super::rust_legacy::wasm_output_path(metadata, checkout).unwrap_err();
        assert!(format!("{err:?}").contains("manifest path"), "{err:?}");
        assert!(
            format!("{err:?}").contains("resolves outside the source checkout"),
            "{err:?}"
        );
        assert!(!base.join("outside/target").exists());
    }
}
