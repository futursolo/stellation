mod cli;
mod manifest;

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Parser;
use cli::{Cli, Command};
use manifest::Manifest;
use tokio::fs;
use tokio::signal::ctrl_c;
use tokio::time::sleep;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
struct Stackctl {
    cli: Arc<Cli>,
    manifest: Arc<Manifest>,
}

impl Stackctl {
    async fn workspace_dir(&self) -> Result<PathBuf> {
        self.cli
            .manifest_path
            .canonicalize()?
            .parent()
            .context("failed to find workspace directory")
            .map(|m| m.to_owned())
    }

    /// Creates and returns the path of the data directory.
    ///
    /// This is `.stackable` in the same parent directory as `stackable.toml`.
    async fn data_dir(&self) -> Result<PathBuf> {
        let data_dir = self.workspace_dir().await?.join(".stackable");

        fs::create_dir_all(&data_dir)
            .await
            .context("failed to create data directory")?;

        Ok(data_dir)
    }

    async fn run_serve(&self, open: bool) -> Result<()> {
        use tokio::process::Command;

        let workspace_dir = self.workspace_dir().await?;
        let data_dir = self.data_dir().await?;
        let dev_server_build_dir = data_dir.join("dev-server-build");
        fs::create_dir_all(&dev_server_build_dir)
            .await
            .context("failed to create build directory for dev server build.")?;

        let trunk_status = Command::new("trunk")
            .arg("build")
            .arg("--dist")
            .arg(&dev_server_build_dir)
            .arg(workspace_dir.join("index.html"))
            .current_dir(&workspace_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()
            .await?;
        if !trunk_status.success() {
            bail!("trunk failed with status {}", trunk_status);
        }

        let build_status = Command::new("cargo")
            .arg("build")
            .arg("--bin")
            .arg(&self.manifest.dev_server.bin_name)
            .current_dir(&workspace_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?
            .wait()
            .await
            .context("failed to build backend")?;

        if !build_status.success() {
            bail!("build failed with status {}", build_status);
        }

        let http_listen_addr = format!("http://{}/", self.manifest.dev_server.listen);

        tracing::info!(
            "Stackable development server is started at: {}",
            http_listen_addr
        );

        let _server_proc = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg(&self.manifest.dev_server.bin_name)
            .current_dir(&workspace_dir)
            .env("STACKCTL_LISTEN_ADDR", &self.manifest.dev_server.listen)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;

        // TODO: wait until the backend connects.
        sleep(Duration::from_secs(1)).await;

        if open {
            Command::new("open")
                .arg(&http_listen_addr)
                .current_dir(&workspace_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
                .wait()
                .await
                .context("failed to open url")?;
        }

        ctrl_c().await?;

        Ok(())
    }

    async fn run(&self) -> Result<()> {
        match self.cli.command {
            Command::Serve { open } => {
                self.run_serve(open).await?;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_env("STACKCTL_LOG"))
        .init();

    let cli = Cli::parse();
    let manifest = cli.load_manifest().await?;

    Stackctl {
        cli: cli.into(),
        manifest,
    }
    .run()
    .await?;

    Ok(())
}
