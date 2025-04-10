pub mod rust_legacy {
    use std::str::FromStr;

    use eyre::{Context, ContextCompat};

    use crate::{
        logic::nep330_build::output::common,
        types::{
            contract_source_metadata::ContractSourceMetadata,
            internal::legacy_rust::{
                manifest_path::MANIFEST_FILE_NAME, CrateMetadata, ManifestPath,
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
        let manifest_path = {
            let manifest_path = manifest_path(contract_source_metadata, contract_source_workdir)?;
            ManifestPath::try_from(manifest_path).wrap_err("Assumption about compiling a rust crate in docker container is invalid: manifest file not found")?
        };

        let crate_metadata = CrateMetadata::collect(manifest_path, false)?;

        let path = crate_metadata.get_legacy_cargo_near_output_path()?;
        tracing::info!(
            target: "near_teach_me",
            parent: &tracing::Span::none(),
            "assumed artifact result path for a rust crate docker build: `{}`", path
        );

        common::path_sane_check(&path, "rust crate")?;

        Ok(path)
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
        let wasm_path = contract_source_workdir.join(relative_path);
        common::path_sane_check(&wasm_path, "generic nep330 1.3.0 compliant")?;
        Ok(wasm_path)
    }
}

mod common {
    use crate::types::internal::legacy_rust::metadata::EXPECTED_EXTENSION;

    pub(super) fn path_sane_check(
        path: &camino::Utf8PathBuf,
        descriptor: &str,
    ) -> eyre::Result<()> {
        if !path.exists() {
            return Err(eyre::eyre!(
                "assumed artifact result path for a {} docker build not found: `{}`",
                descriptor,
                path
            ));
        }
        if !path.is_file() {
            return Err(eyre::eyre!("result path isn't a file: `{}`", path));
        }
        // this check is redundant due to [CrateMetadata::get_legacy_cargo_near_output_path]
        // but keeping it here for future duplication
        if path.extension() != Some(EXPECTED_EXTENSION) {
            return Err(eyre::eyre!(
                "result path doesn't have a `wasm` extension: `{}`",
                path
            ));
        }
        Ok(())
    }
}
