use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tokio::fs;

use crate::manifest::Manifest;

#[derive(Parser, Debug)]
pub(crate) struct ServeCommand {
    /// Open browser after the development server is ready.
    #[arg(long)]
    pub open: bool,
    /// The name of the env profile. [Default: the same name as the build profile]
    #[arg(long)]
    pub env: Option<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct BuildCommand {
    /// Build artifacts in release mode, with optimizations.
    #[arg(long)]
    pub release: bool,
    /// The name of the env profile. [Default: the same name as the build profile]
    #[arg(long)]
    pub env: Option<String>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CliCommand {
    /// Start the development server, serve backend and frontend, watch file changes and
    /// rebuild if needed.
    Serve(ServeCommand),
    /// Build the server and client for final distribution.
    Build(BuildCommand),
}

#[derive(Parser, Debug)]
pub(crate) struct Cli {
    /// The path to the manifest file.
    ///
    /// If you omit this value, it will load from current working directory.
    #[arg(short, long, value_name = "FILE", default_value = "stellation.toml")]
    pub manifest_path: PathBuf,

    #[command(subcommand)]
    pub command: CliCommand,
}

impl Cli {
    pub async fn load_manifest(&self) -> Result<Arc<Manifest>> {
        let manifest_str = fs::read_to_string(&self.manifest_path).await.context(
            "failed to load manifest, do you have stellation.toml in the current directory?",
        )?;

        toml::from_str(&manifest_str)
            .map(Arc::new)
            .context("failed to parse stellation.toml")
    }
}
