use std::str::FromStr;

use crate::types::whitelist::Whitelist;
pub const DOCKER_IMAGE_REGEX_PATTERN: &str =
    r#"^(?P<image>[^:@\s]+?)(?::(?P<tag>[^@\s]+?))?(@sha256:(?P<digest>[a-f0-9]{64}))$"#;

/// TODO #H2: check [super::BuildInfo::build_environment] for match with one of[WhitelistEntry::expected_command_prefix] (if w[crate::types::whitelist::Whitelist]hitelist argument [Option::is_some])
/// TODO #H3: add validation for[super::BuildInfo::build_command], that the vec isn't empty, and all tokens aren't empty
/// TODO #H4: add validation of [super::BuildInfo::build_command] with [crate::types::whitelist::WhitelistEntry::expected_command_prefix] argument on matching [crate::types::whitelist::WhitelistEntry::image_org_prefix] (if [crate::types::whitelist::Whitelist] argument [Option::is_some])
impl super::ContractSourceMetadata {
    #[allow(unused_variables)]
    pub fn validate(&self, whitelist: Option<Whitelist>) -> eyre::Result<()> {
        if self.build_info.is_none() {
            return Err(eyre::eyre!(
                "`build_info` field of `ContractSourceMetadata` cannot be null"
            ));
        }

        let build_info = self.build_info.as_ref().unwrap();

        build_info.validate_contract_path()?;
        build_info.validate_build_environment(whitelist.as_ref())?;

        Ok(())
    }
}

impl super::build_info::BuildInfo {
    #[allow(unused_variables)]
    pub fn validate_build_environment(&self, whitelist: Option<&Whitelist>) -> eyre::Result<()> {
        let regex = regex::Regex::new(DOCKER_IMAGE_REGEX_PATTERN).expect("no error");

        if !regex.is_match(&self.build_environment) {
            return Err(eyre::eyre!(
                "`{}` doesn't match {}",
                self.build_environment,
                DOCKER_IMAGE_REGEX_PATTERN
            ));
        }
        Ok(())
    }
    pub fn validate_contract_path(&self) -> eyre::Result<()> {
        match unix_path::PathBuf::from_str(&self.contract_path) {
            Err(err) => {
                return Err(eyre::eyre!(
                    "`contract_path` field (`{}`) of `BuildInfo` isn't a valid unix path: {:#?}",
                    self.contract_path,
                    err,
                ));
            }
            Ok(path) => {
                if !path.is_relative() {
                    return Err(eyre::eyre!(
                        "`contract_path` field (`{}`) of `BuildInfo` isn't a relative unix path",
                        self.contract_path,
                    ));
                }
                for component in path.components() {
                    let unix_str = component.as_unix_str();
                    if let Err(err) = unix_str.to_owned().into_string() {
                        // this is somewhat impossible to reach, as the whole path was parsed from a [String]
                        return Err(eyre::eyre!(
                            "`contract_path` field (`{}`) of `BuildInfo` contains a component which is not a valid utf8 string: `{:?}",
                            self.contract_path,
                            err,
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::contract_source_metadata::validate::DOCKER_IMAGE_REGEX_PATTERN;

    #[test]
    fn check_regex() {
        let regex = regex::Regex::new(DOCKER_IMAGE_REGEX_PATTERN).expect("no error");
        let right_haystack = "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2";

        assert!(regex.is_match(right_haystack));

        let wrong_haystack_a = " sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2";
        assert!(!regex.is_match(wrong_haystack_a));

        let wrong_haystack_b = "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2 ";
        assert!(!regex.is_match(wrong_haystack_b));
    }
}
