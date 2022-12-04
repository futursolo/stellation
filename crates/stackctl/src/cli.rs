use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tokio::fs;

use crate::manifest::Manifest;

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Start the development server, serve backend and frontend, watch file changes and
    /// rebuild if needed.
    Serve {
        /// Open browser after the development server is ready.
        #[arg(long)]
        open: bool,
    },
}

#[derive(Parser, Debug)]
pub(crate) struct Cli {
    /// The path to the manifest file.
    ///
    /// If you omit this value, it will load from current working directory.
    #[arg(short, long, value_name = "FILE", default_value = "stackable.toml")]
    pub manifest_path: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub async fn load_manifest(&self) -> Result<Arc<Manifest>> {
        let manifest_str = fs::read_to_string(&self.manifest_path).await.context(
            "failed to load manifest, do you have stackable.toml in the current directory?",
        )?;

        toml::from_str(&manifest_str)
            .map(Arc::new)
            .context("failed to parse stackable.toml")
    }
}
