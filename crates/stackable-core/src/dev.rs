use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackctlMetadata {
    pub listen_addr: String,
    pub frontend_dev_build_dir: PathBuf,
}

impl StackctlMetadata {
    pub const ENV_NAME: &str = "STACKCTL_METADATA";

    pub fn from_json(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}
