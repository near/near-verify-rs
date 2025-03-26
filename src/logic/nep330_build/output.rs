mod rust_legacy {
    use std::str::FromStr;

    use eyre::Context;

    use crate::types::{
        contract_source_metadata::ContractSourceMetadata,
        internal::legacy_rust::{
            CrateMetadata, ManifestPath, manifest_path::MANIFEST_FILE_NAME,
            metadata::EXPECTED_EXTENSION,
        },
    };

    fn manifest_path(
        contract_source_metadata: ContractSourceMetadata,
        contract_source_workdir: camino::Utf8PathBuf,
    ) -> camino::Utf8PathBuf {
        let contract_path = {
            let contract_path = contract_source_metadata
                .build_info
                .expect(
                    "cannot be [Option::None] as per [ContractSourceMetadata::validate_meta] check",
                )
                .contract_path;
            unix_path::PathBuf::from_str(&contract_path).expect(
                "should be a valid relative [unix_path::PathBuf] as per [ContractSourceMetadata::validate_meta] check",
            )
        };
        let components = {
            contract_path.components().map(|component| {
                let unix_str = component.as_unix_str();
                let string = unix_str
                    .to_owned()
                    .into_string()
                    .expect("should be a valid utf8 [String] component as per [ContractSourceMetadata::validate_meta] check");
                camino::Utf8PathBuf::from(string)
            })
        };
        let mut path = contract_source_workdir.clone();
        path.extend(components);
        path.push(MANIFEST_FILE_NAME);
        path
    }

    pub fn wasm_output_path(
        contract_source_metadata: ContractSourceMetadata,
        contract_source_workdir: camino::Utf8PathBuf,
    ) -> eyre::Result<camino::Utf8PathBuf> {
        let manifest_path = {
            let manifest_path = manifest_path(contract_source_metadata, contract_source_workdir);
            ManifestPath::try_from(manifest_path).wrap_err("Assumption about compiling a rust crate in docker container is invalid: manifest file not found")?
        };

        let crate_metadata = CrateMetadata::collect(manifest_path, false)?;

        let path = crate_metadata.get_legacy_cargo_near_output_path()?;
        tracing::info!(
            target: "near_teach_me",
            parent: &tracing::Span::none(),
            "assumed artifact result path for a rust crate docker build: `{}`", path
        );
        if !path.exists() {
            return Err(eyre::eyre!(
                "assumed artifact result path for a rust crate docker build not found: `{}`",
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
        Ok(path)
    }
}

pub use rust_legacy::wasm_output_path as rust_legacy_wasm_output_path;
