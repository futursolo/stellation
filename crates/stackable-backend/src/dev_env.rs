use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DevEnv {
    pub listen_addr: String,
    pub dev_server_build_path: PathBuf,
}

impl DevEnv {
    pub fn from_env() -> Result<Option<Self>> {
        let encoded = match env::var("STACKCTL_DEV_ENV") {
            Ok(m) => m,
            Err(_) => return Ok(None),
        };

        let decoded = serde_json::from_str(&encoded).context("failed to decoded dev env")?;

        Ok(Some(decoded))
    }

    pub fn set_env(&self, process: &mut Command) -> Result<()> {
        let encoded = serde_json::to_string(&self).context("failed to encode dev env")?;
        process.env("STACKCTL_DEV_ENV", &encoded);

        Ok(())
    }
}
