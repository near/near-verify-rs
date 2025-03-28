use std::str::FromStr;

use crate::types::whitelist::{Whitelist, WhitelistEntry};

use super::BuildInfo;
pub const DOCKER_IMAGE_REGEX_PATTERN: &str =
    r#"^(?P<image>[^:@\s]+?)(?::(?P<tag>[^@\s]+?))?(@sha256:(?P<digest>[a-f0-9]{64}))$"#;

impl super::ContractSourceMetadata {
    pub fn validate(&self, whitelist: Option<Whitelist>) -> eyre::Result<()> {
        if self.build_info.is_none() {
            return Err(eyre::eyre!(
                "`build_info` field of `ContractSourceMetadata` cannot be null"
            ));
        }

        let build_info = self.build_info.as_ref().unwrap();

        build_info.validate_contract_path()?;
        build_info.validate_build_command_basic()?;

        let image = build_info.validate_build_env_on_regex()?;
        if let Some(whitelist) = whitelist {
            let entry = BuildInfo::validate_build_image_on_whitelist(&image, whitelist)?;

            build_info.validate_build_command_on_whitelist(entry)?;
        }

        Ok(())
    }
}

impl super::build_info::BuildInfo {
    pub fn validate_build_env_on_regex(&self) -> eyre::Result<String> {
        let regex = regex::Regex::new(DOCKER_IMAGE_REGEX_PATTERN).expect("no error");

        if !regex.is_match(&self.build_environment) {
            return Err(eyre::eyre!(
                "`{}` doesn't match {}",
                self.build_environment,
                DOCKER_IMAGE_REGEX_PATTERN
            ));
        }
        let image = regex
            .captures(&self.build_environment)
            .and_then(|captures| captures.name("image"));

        image
            .map(|capture| capture.as_str().to_string())
            .ok_or(eyre::eyre!(
                "`{}` didn't match any `image` group in {}",
                self.build_environment,
                DOCKER_IMAGE_REGEX_PATTERN
            ))
    }
    pub fn validate_build_image_on_whitelist(
        image: &str,
        whitelist: Whitelist,
    ) -> eyre::Result<WhitelistEntry> {
        let entry_match = whitelist
            .iter()
            .find(|entry| entry.expected_docker_image == image)
            .cloned();

        entry_match.ok_or(eyre::eyre!(
            "no matching entry found for `{}` in whitelist : {:?}",
            image,
            whitelist
        ))
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
    pub fn validate_build_command_basic(&self) -> eyre::Result<()> {
        if self.build_command.is_empty() {
            return Err(eyre::eyre!("empty {:?} build command", self.build_command));
        }

        for token in self.build_command.iter() {
            if token.is_empty() {
                return Err(eyre::eyre!("empty token {:?} in build command", token));
            }
        }
        Ok(())
    }

    pub fn validate_build_command_on_whitelist(&self, entry: WhitelistEntry) -> eyre::Result<()> {
        let expected_cmd_len = entry.expected_command_prefix.len();
        if (self.build_command.len() < expected_cmd_len)
            || (self.build_command[1..expected_cmd_len]
                != entry.expected_command_prefix[1..expected_cmd_len])
        {
            return Err(eyre::eyre!(
                "build_command {:?} must start with expected whitelist command prefix {:?}",
                self.build_command,
                entry.expected_command_prefix
            ));
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

        let image = regex
            .captures(right_haystack)
            .and_then(|captures| captures.name("image"));
        assert_eq!("sourcescan/cargo-near", image.expect("to be some").as_str());

        let wrong_haystack_a = " sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2";
        assert!(!regex.is_match(wrong_haystack_a));

        let wrong_haystack_b = "sourcescan/cargo-near:0.13.4-rust-1.85.0@sha256:a9d8bee7b134856cc8baa142494a177f2ba9ecfededfcdd38f634e14cca8aae2 ";
        assert!(!regex.is_match(wrong_haystack_b));
    }
}
