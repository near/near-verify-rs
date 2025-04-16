use serde::{Deserialize, Serialize};

pub type Whitelist = Vec<WhitelistEntry>;

#[derive(Debug, Clone, PartialEq, Default, Eq, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub expected_docker_image: String,
}
