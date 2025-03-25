// ====================== NEP-330 1.2.0 - Build Details Extension ===========
/// NEP-330 1.2.0
pub const BUILD_ENVIRONMENT: &str = "NEP330_BUILD_INFO_BUILD_ENVIRONMENT";
/// NEP-330 1.2.0
pub const BUILD_COMMAND: &str = "NEP330_BUILD_INFO_BUILD_COMMAND";
/// NEP-330 1.2.0
pub const CONTRACT_PATH: &str = "NEP330_BUILD_INFO_CONTRACT_PATH";
/// NEP-330 1.2.0
pub const SOURCE_CODE_SNAPSHOT: &str = "NEP330_BUILD_INFO_SOURCE_CODE_SNAPSHOT";
// ====================== End section =======================================

// ====================== NEP-330 1.1.0 - Contract Metadata Extension ===========
/// NEP-330 1.1.0
pub const LINK: &str = "NEP330_LINK";
/// NEP-330 1.1.0
pub const VERSION: &str = "NEP330_VERSION";
// ====================== End section =======================================
pub mod nonspec {
    pub const SERVER_DISABLE_INTERACTIVE: &str = "CARGO_NEAR_SERVER_BUILD_DISABLE_INTERACTIVE";
}
