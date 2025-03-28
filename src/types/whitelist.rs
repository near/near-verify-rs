use serde::{Deserialize, Serialize};

pub type Whitelist = Vec<WhitelistEntry>;

#[derive(Debug, Clone, PartialEq, Default, Eq, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub image_org_prefix: String,
    pub expected_command_prefix: Vec<String>,
}
