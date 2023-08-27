//! The stellation command utility.
//!
//! This crate is vendored as `stctl` in the templates.

#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]
#![deny(non_snake_case)]
#![deny(clippy::cognitive_complexity)]
#![deny(missing_docs)]
#![cfg_attr(documenting, feature(doc_cfg))]
#![cfg_attr(documenting, feature(doc_auto_cfg))]
#![cfg_attr(any(releasing, not(debug_assertions)), deny(dead_code, unused_imports))]

mod builder;
mod cli;
mod env_file;
mod indicators;
mod manifest;
mod paths;
mod profile;
mod utils;

use std::path::PathBuf;
use std::pin::pin;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use clap::Parser;
use cli::{BuildCommand, Cli, CliCommand, ServeCommand};
use console::{style, Term};
use env_file::EnvFile;
use futures::future::ready;
use futures::stream::unfold;
use futures::{FutureExt, Stream, StreamExt};
use manifest::Manifest;
use notify::{recommended_watcher, Event, RecursiveMode, Watcher};
use paths::Paths;
use profile::Profile;
use stellation_core::dev::StctlMetadata;
use tokio::fs;
use tokio::process::Child;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::spawn_blocking;
use tokio::time::sleep;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::Level;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::builder::Builder;
use crate::indicators::ServeProgress;

#[derive(Debug)]
struct Stctl {
    cli: Arc<Cli>,
    paths: Arc<Paths>,
    manifest: Arc<Manifest>,
    profile: Profile,
    env_file: EnvFile,
}

impl Stctl {
    async fn new(cli: Cli) -> Result<Self> {
        let manifest = cli.load_manifest().await?;

        let profile = match cli.command {
            CliCommand::Serve(_) => Profile::new_debug(),
            CliCommand::Build(BuildCommand { release, .. }) => {
                if release {
                    Profile::new_release()
                } else {
                    Profile::new_debug()
                }
            }
            CliCommand::Clean => Profile::new_debug(),
        };

        let env_name = match cli.command {
            CliCommand::Build(BuildCommand {
                env: Some(ref m), ..
            })
            | CliCommand::Serve(ServeCommand {
                env: Some(ref m), ..
            }) => m,
            _ => profile.name(),
        };

        let env_file = EnvFile::new(env_name);
        let paths = Paths::new(&cli.manifest_path).await?;

        Ok(Self {
            cli: cli.into(),
            paths: paths.into(),
            manifest,
            profile,
            env_file,
        })
    }

    async fn watch_changes(&self) -> Result<impl Stream<Item = SystemTime>> {
        let workspace_dir = self.paths.workspace_dir().await?;
        let (tx, rx) = unbounded_channel::<PathBuf>();

        let mut watcher = recommended_watcher(move |e: Result<Event, _>| {
            if let Ok(e) = e {
                for path in e.paths {
                    if tx.send(path).is_err() {
                        break;
                    }
                }
            }
        })
        .context("failed to watch workspace changes")?;

        watcher
            .watch(workspace_dir, RecursiveMode::Recursive)
            .context("failed to watch workspace")?;

        let stream = UnboundedReceiverStream::new(rx)
            .filter(|p| {
                let p_str = p.as_os_str().to_string_lossy();
                if p_str.contains("target/") {
                    return ready(false);
                }
                if p_str.contains(".stellation/") {
                    return ready(false);
                }
                if !p_str.contains("src/") {
                    return ready(false);
                }

                ready(true)
            })
            .boxed();

        Ok(unfold(
            (stream, watcher),
            |(mut stream, watcher)| async move {
                // We wait until first item is available.
                stream.next().await?;

                let mut sleep_fur = pin!(sleep(Duration::from_millis(100)).fuse());

                // This makes sure we filter all items between first item and sleep completes,
                // whilst still returns at least 1 item at the end of the period.
                loop {
                    let mut next_path_fur = pin!(stream.next().fuse());

                    futures::select! {
                        _ = sleep_fur => break,
                        _ = next_path_fur => {},
                    }
                }

                Some((SystemTime::now(), (stream, watcher)))
            },
        ))
    }

    async fn open_browser(&self, http_listen_addr: &str) -> Result<()> {
        if let Err(e) = webbrowser::open(http_listen_addr) {
            tracing::warn!("stctl was unable to open the browser");
            tracing::debug!("due to: {:?}", e);
        }

        Ok(())
    }

    async fn serve_once(&self) -> Result<Child> {
        use tokio::process::Command;

        let http_listen_addr = format!("http://{}/", self.manifest.dev_server.listen);

        let builder = Builder::new(self).await?.watch_build(true);

        let bar = ServeProgress::new();

        let workspace_dir = self.paths.workspace_dir().await?;
        bar.step_build_frontend();
        let frontend_build_dir = builder.build_frontend().await?;

        bar.step_build_backend();
        let backend_build_path = builder.build_backend().await?;

        let meta = StctlMetadata {
            listen_addr: self.manifest.dev_server.listen.to_string(),
            frontend_dev_build_dir: frontend_build_dir.to_owned(),
        };

        bar.step_starting();

        let envs = self.env_file.load(workspace_dir);

        let server_proc = Command::new(&backend_build_path)
            .current_dir(workspace_dir)
            .envs(envs)
            .env(StctlMetadata::ENV_NAME, meta.to_json()?)
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

        bar.hide();

        Ok(server_proc)
    }

    async fn run_serve(&self, cmd_args: &ServeCommand) -> Result<()> {
        let changes = self.watch_changes().await?;
        let mut changes = pin!(changes);

        let mut first_run = true;

        'outer: loop {
            let start_time = SystemTime::now();
            let http_listen_addr = format!("http://{}/", self.manifest.dev_server.listen);

            let server_proc = match self.serve_once().await {
                Ok(server_proc) => {
                    let time_taken_in_f64 =
                        f64::try_from(i32::try_from(start_time.elapsed()?.as_millis())?)? / 1000.0;

                    Term::stderr().clear_screen()?;

                    eprintln!(
                        "{}",
                        style(format!("Built in {time_taken_in_f64:.2}s!"))
                            .green()
                            .bold()
                    );
                    eprintln!("Stellation development server has started!");
                    eprintln!();
                    eprintln!();
                    eprintln!("    Listening at: {http_listen_addr}");
                    eprintln!();
                    eprintln!();
                    eprintln!(
                        "{} This build is not optimised and should not be used in production.",
                        style("Note:").yellow().bold()
                    );
                    eprintln!(
                        "To produce a production build, you can use `{}`.",
                        style("cargo make build").cyan().bold()
                    );

                    Some(server_proc)
                }
                Err(e) => {
                    tracing::error!("failed to build development server: {:?}", e);
                    None
                }
            };

            if cmd_args.open && first_run {
                self.open_browser(&http_listen_addr).await?;
            }

            first_run = false;

            'inner: loop {
                match changes.next().await {
                    Some(change_time) => {
                        if change_time > start_time {
                            break 'inner;
                        }
                    }
                    None => break 'outer,
                }
            }

            if let Some(mut m) = server_proc {
                m.kill().await.context("failed to stop server")?;
            }
        }

        Ok(())
    }

    async fn run_build(&self, _cmd_args: &BuildCommand) -> Result<()> {
        let target_name = self.profile.name();

        eprintln!(
            "{}",
            style(format!("Building with {target_name} profile..."))
                .cyan()
                .bold()
        );

        let start_time = SystemTime::now();

        let build_dir = self.paths.build_dir().await?;

        let builder = Builder::new(self).await?;

        let frontend_artifact_dir = builder.build_frontend().await?;
        let backend_artifact_dir = builder.backend_build_dir().await?;
        let backend_artifact_path = builder.build_backend().await?;

        let backend_build_dir = build_dir.join("backend");
        let frontend_build_dir = build_dir.join("frontend");

        if backend_build_dir.exists() {
            fs::remove_dir_all(&backend_build_dir)
                .await
                .context("failed to clean past backend builds.")?;
        }

        fs::create_dir_all(&backend_build_dir)
            .await
            .context("failed to create backend build directory.")?;

        if frontend_build_dir.exists() {
            fs::remove_dir_all(&frontend_build_dir)
                .await
                .context("failed to clean past frontend builds.")?;
        }

        fs::create_dir_all(&frontend_build_dir)
            .await
            .context("failed to create frontend build directory.")?;

        fs::copy(
            &backend_artifact_path,
            backend_build_dir.join(
                backend_artifact_path
                    .file_name()
                    .context("failed to find backend binary name")?,
            ),
        )
        .await
        .context("failed to copy backend")?;

        {
            let frontend_artifact_dir = frontend_artifact_dir.to_owned();
            let frontend_build_dir = frontend_build_dir.to_owned();

            spawn_blocking(move || {
                use fs_extra::dir::{copy, CopyOptions};

                copy(
                    frontend_artifact_dir,
                    frontend_build_dir,
                    &CopyOptions::new(),
                )
            })
        }
        .await
        .context("failed to copy frontend")?
        .context("failed to copy frontend")?;

        fs::remove_dir_all(backend_artifact_dir)
            .await
            .context("failed to remove backend temporary artifacts.")?;
        fs::remove_dir_all(frontend_artifact_dir)
            .await
            .context("failed to remove frontend temporary artifacts.")?;

        let time_taken_in_f64 =
            f64::try_from(i32::try_from(start_time.elapsed()?.as_millis())?)? / 1000.0;
        eprintln!(
            "{}",
            style(format!("Built in {time_taken_in_f64:.2}s!"))
                .green()
                .bold()
        );
        eprintln!("The artifact is available at: {}", build_dir.display());

        Ok(())
    }

    async fn run_clean(&self) -> Result<()> {
        use tokio::process::Command;

        let workspace_dir = self.paths.workspace_dir().await?;
        let build_dir = self.paths.build_dir().await?;
        let data_dir = self.paths.data_dir().await?;

        let envs = self.env_file.load(workspace_dir);

        let start_time = SystemTime::now();

        Command::new("cargo")
            .arg("clean")
            .current_dir(workspace_dir)
            .envs(envs)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?
            .wait_with_output()
            .await
            .context("failed to clean cargo data")?;

        fs::remove_dir_all(&build_dir)
            .await
            .context("failed to clean build dir")?;
        fs::remove_dir_all(&data_dir)
            .await
            .context("failed to clean data dir")?;

        let time_taken_in_f64 =
            f64::try_from(i32::try_from(start_time.elapsed()?.as_millis())?)? / 1000.0;

        eprintln!(
            "{}",
            style(format!(
                "Build artifact and temporary files cleared in {time_taken_in_f64:.2}s!"
            ))
            .green()
            .bold()
        );

        Ok(())
    }

    async fn run(&self) -> Result<()> {
        match self.cli.command {
            CliCommand::Serve(ref m) => {
                self.run_serve(m).await?;
            }
            CliCommand::Build(ref m) => {
                self.run_build(m).await?;
            }
            CliCommand::Clean => {
                self.run_clean().await?;
            }
        }

        Ok(())
    }
}

/// Runs stctl.
///
/// This is the main function for a vendored copy of stctl.
pub async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .with_env_var("STCTL_LOG")
                .from_env_lossy(),
        )
        .init();

    let cli = Cli::parse();
    Stctl::new(cli).await?.run().await?;

    Ok(())
}
