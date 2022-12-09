mod cli;
mod indicators;
mod manifest;
mod utils;

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use clap::Parser;
use cli::{Cli, Command};
use console::{style, Term};
use manifest::Manifest;
use stackable_backend::DevEnv;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::signal::ctrl_c;
use tokio::time::sleep;
use tokio::{fs, spawn};
use tracing::Level;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::indicators::ServeProgress;
use crate::utils::random_str;

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

    async fn frontend_data_dir(&self) -> Result<PathBuf> {
        let frontend_data_dir = self.data_dir().await?.join("frontend");

        fs::create_dir_all(&frontend_data_dir)
            .await
            .context("failed to create frontend data directory")?;

        Ok(frontend_data_dir)
    }

    async fn backend_data_dir(&self) -> Result<PathBuf> {
        let backend_data_dir = self.data_dir().await?.join("backend");

        fs::create_dir_all(&backend_data_dir)
            .await
            .context("failed to create backend data directory")?;

        Ok(backend_data_dir)
    }

    async fn transfer_to_file<R, P>(source: R, target: P) -> Result<()>
    where
        R: 'static + AsyncRead + Send,
        P: Into<PathBuf>,
    {
        let target_path = target.into();
        let mut target = fs::File::create(&target_path)
            .await
            .with_context(|| format!("failed to create {}", target_path.display()))?;

        let inner = async move {
            tokio::pin!(source);

            loop {
                let mut buf = [0_u8; 8192];
                let buf_len = source.read(&mut buf[..]).await?;

                if buf_len == 0 {
                    break;
                }
                target.write_all(&buf[..buf_len]).await?;
            }

            Ok::<(), anyhow::Error>(())
        };

        spawn(async move {
            if let Err(e) = inner
                .await
                .with_context(|| format!("failed to transfer logs to: {}", target_path.display()))
            {
                tracing::error!("{:#?}", e);
            }
        });

        Ok(())
    }

    async fn build_frontend(&self) -> Result<PathBuf> {
        use tokio::process::Command;

        let frontend_data_dir = self.frontend_data_dir().await?;
        let frontend_build_dir = frontend_data_dir.join("dev-builds").join(random_str()?);
        let workspace_dir = self.workspace_dir().await?;

        fs::create_dir_all(&frontend_build_dir)
            .await
            .context("failed to create build directory for frontend build.")?;

        let mut child = Command::new("trunk")
            .arg("build")
            .arg("--dist")
            .arg(&frontend_build_dir)
            .arg(workspace_dir.join("index.html"))
            .current_dir(&workspace_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(m) = child.stdout.take() {
            Self::transfer_to_file(
                m,
                frontend_data_dir.join(format!("log-stdout-{}", random_str()?)),
            )
            .await?;
        }

        if let Some(m) = child.stderr.take() {
            Self::transfer_to_file(
                m,
                frontend_data_dir.join(format!("log-stderr-{}", random_str()?)),
            )
            .await?;
        }

        let status = child.wait().await?;

        // We try again with logs printed to console.
        if !status.success() {
            let mut child = Command::new("trunk")
                .arg("build")
                .arg("--dist")
                .arg(&frontend_build_dir)
                .arg(workspace_dir.join("index.html"))
                .current_dir(&workspace_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;

            let status = child.wait().await?;

            if !status.success() {
                bail!("trunk failed with status {}", status);
            }
        }

        Ok(frontend_build_dir)
    }

    async fn build_backend(&self) -> Result<()> {
        use tokio::process::Command;

        let backend_data_dir = self.backend_data_dir().await?;
        let workspace_dir = self.workspace_dir().await?;

        let mut child = Command::new("cargo")
            .arg("build")
            .arg("--bin")
            .arg(&self.manifest.dev_server.bin_name)
            .current_dir(&workspace_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        if let Some(m) = child.stdout.take() {
            Self::transfer_to_file(
                m,
                backend_data_dir.join(format!("log-stdout-{}", random_str()?)),
            )
            .await?;
        }

        if let Some(m) = child.stderr.take() {
            Self::transfer_to_file(
                m,
                backend_data_dir.join(format!("log-stderr-{}", random_str()?)),
            )
            .await?;
        }

        let status = child.wait().await?;

        // We try again with logs printed to console.
        if !status.success() {
            let mut child = Command::new("cargo")
                .arg("build")
                .arg("--bin")
                .arg(&self.manifest.dev_server.bin_name)
                .current_dir(&workspace_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .kill_on_drop(true)
                .spawn()?;

            let status = child.wait().await?;

            if !status.success() {
                bail!("trunk failed with status {}", status);
            }
        }

        Ok(())
    }

    async fn open_browser(&self, http_listen_addr: &str) -> Result<()> {
        use tokio::process::Command;
        let workspace_dir = self.workspace_dir().await?;

        Command::new("open")
            .arg(http_listen_addr)
            .current_dir(&workspace_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
            .wait()
            .await
            .context("failed to open url")?;

        Ok(())
    }

    async fn run_serve(&self, open: bool) -> Result<()> {
        use tokio::process::Command;

        let start_time = SystemTime::now();
        let http_listen_addr = format!("http://{}/", self.manifest.dev_server.listen);

        let bar = ServeProgress::new();

        let workspace_dir = self.workspace_dir().await?;
        bar.step_build_frontend();
        let frontend_build_dir = self.build_frontend().await?;

        bar.step_build_backend();
        self.build_backend().await?;

        let dev_env = DevEnv {
            listen_addr: self.manifest.dev_server.listen.to_string(),
            dev_server_build_path: frontend_build_dir.clone(),
        };

        bar.step_starting();
        let mut server_cmd = Command::new("cargo");
        dev_env.set_env(&mut server_cmd)?;

        let _server_proc = server_cmd
            .arg("run")
            .arg("--bin")
            .arg(&self.manifest.dev_server.bin_name)
            .current_dir(&workspace_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;

        while reqwest::ClientBuilder::default()
            .timeout(Duration::from_secs(1))
            .build()?
            .get(&http_listen_addr)
            .send()
            .await
            .and_then(|m| m.error_for_status())
            .is_err()
        {
            sleep(Duration::from_secs(1)).await;
        }

        if open {
            self.open_browser(&http_listen_addr).await?;
        }

        bar.hide();

        let time_taken_in_f64 =
            f64::try_from(i32::try_from(start_time.elapsed()?.as_millis())?)? / 1000.0;

        Term::stderr().clear_screen()?;

        eprintln!(
            "{}",
            style(format!("Built in {:.2}s!", time_taken_in_f64))
                .green()
                .bold()
        );
        eprintln!("Stackable development server has started!");
        eprintln!();
        eprintln!();
        eprintln!("    Listen: {}", http_listen_addr);
        eprintln!();
        eprintln!();
        eprintln!(
            "To produce a production build, you can use `{}`",
            style("stackctl build --release").cyan().bold()
        );

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
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .with_env_var("STACKCTL_LOG")
                .from_env_lossy(),
        )
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
