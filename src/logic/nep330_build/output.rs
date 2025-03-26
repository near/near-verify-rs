mod rust_legacy {
    use std::str::FromStr;

    use crate::types::{
        contract_source_metadata::ContractSourceMetadata,
        internal::legacy_rust::manifest_path::MANIFEST_FILE_NAME,
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
        unimplemented!();
        // println!(
        //     " {} {}",
        //     "artifact search location in temporary build site:".green(),
        //     tmp_out_dir
        // );

        // let filename = format!("{}.wasm", tmp_crate_metadata.formatted_package_name());

        // let in_wasm_path = tmp_out_dir.join(filename.clone());
        // if !in_wasm_path.exists() {
        //     return Err(eyre::eyre!(
        //         "Temporary build site result wasm file not found: `{:?}`.",
        //         in_wasm_path
        //     ));
        // }
        // if !in_wasm_path.is_file() {
        //     return Err(eyre::eyre!(
        //         "result path isn't a file: `{:?}`.",
        //         in_wasm_path
        //     ));
        // }
        // if !in_wasm_path.extension() != "wasm" {
        //     return Err(eyre::eyre!(
        //         "result path isn't a file: `{:?}`.",
        //         in_wasm_path
        //     ));
        // }
    }
}

pub const EXPECTED_EXTENSION: &str = "WASM";
pub use rust_legacy::wasm_output_path as rust_legacy_wasm_output_path;
