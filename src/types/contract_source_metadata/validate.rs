use std::str::FromStr;

/// TODO #H4: add validation of [BuildInfoMixed::build_environment] with `images_whitelist` [Vec<String>] argument
/// TODO #H3: check [BuildInfoMixed::build_environment] for regex match
/// TODO #H2: add validation for `build_command`, that the vec isn't empty, and all tokens aren't empty
/// TODO #C1: extract a [ContractSourceMetadata::validate_contract_path] method
impl super::ContractSourceMetadata {
    pub fn validate(&self) -> eyre::Result<()> {
        if self.build_info.is_none() {
            return Err(eyre::eyre!(
                "`build_info` field of `ContractSourceMetadata` cannot be null"
            ));
        }

        let build_info = self.build_info.as_ref().unwrap();
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
}
