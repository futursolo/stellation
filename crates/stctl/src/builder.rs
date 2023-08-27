use std::path::{Path, PathBuf};
use std::pin::pin;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use cargo_metadata::Metadata;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::sync::OnceCell;
use tokio::{fs, spawn};

use crate::env_file::EnvFile;
use crate::manifest::Manifest;
use crate::paths::Paths;
use crate::profile::Profile;
use crate::utils::random_str;
use crate::Stctl;

#[derive(Debug)]
pub(crate) struct Builder {
    build_id: String,
    paths: Arc<Paths>,
    profile: Profile,
    env_file: EnvFile,
    manifest: Arc<Manifest>,

    is_watch_build: bool,

    frontend_build_dir: OnceCell<PathBuf>,
    backend_build_dir: OnceCell<PathBuf>,

    backend_target: Option<String>,
}

impl Builder {
    pub async fn new(stctl: &Stctl) -> Result<Self> {
        Ok(Builder {
            build_id: random_str()?,
            paths: stctl.paths.clone(),
            profile: stctl.profile.clone(),

            env_file: stctl.env_file.clone(),
            manifest: stctl.manifest.clone(),

            is_watch_build: false,

            frontend_build_dir: OnceCell::new(),
            backend_build_dir: OnceCell::new(),

            backend_target: None,
        })
    }

    /// Sets to true for watch build.
    pub fn watch_build(mut self, is_watch_build: bool) -> Self {
        self.is_watch_build = is_watch_build;

        self
    }

    /// Sets the backend target.
    pub fn backend_target(mut self, backend_target: Option<String>) -> Self {
        self.backend_target = backend_target;

        self
    }

    /// Returns the frontend build directory of current build.
    pub async fn frontend_build_dir(&self) -> Result<&Path> {
        self.frontend_build_dir
            .get_or_try_init(|| async {
                let frontend_build_dir =
                    self.paths.frontend_builds_dir().await?.join(&self.build_id);

                fs::create_dir_all(&frontend_build_dir)
                    .await
                    .context("failed to create build directory for frontend build.")?;

                Ok(frontend_build_dir)
            })
            .await
            .map(|m| m.as_ref())
    }

    /// Returns the backend build directory of current build.
    pub async fn backend_build_dir(&self) -> Result<&Path> {
        self.backend_build_dir
            .get_or_try_init(|| async {
                let backend_build_dir = self.paths.backend_builds_dir().await?.join(&self.build_id);

                fs::create_dir_all(&backend_build_dir)
                    .await
                    .context("failed to create build directory for backend build.")?;

                Ok(backend_build_dir)
            })
            .await
            .map(|m| m.as_ref())
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
            let mut source = pin!(source);

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

    pub async fn build_frontend(&self) -> Result<&Path> {
        use tokio::process::Command;

        let frontend_logs_dir = self.paths.frontend_logs_dir().await?;
        let frontend_build_dir = self.frontend_build_dir().await?;
        let workspace_dir = self.paths.workspace_dir().await?;

        let create_proc = || {
            let mut proc = Command::new("trunk");
            proc.arg("build")
                .arg("--dist")
                .arg(frontend_build_dir)
                .arg(workspace_dir.join("index.html"))
                .current_dir(workspace_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            if let Some(m) = self.profile.to_profile_argument() {
                proc.arg(m);
            }

            let envs = self.env_file.load(workspace_dir);
            proc.envs(envs);

            if !self.is_watch_build {
                proc.stdout(Stdio::inherit()).stderr(Stdio::inherit());
            }
            proc
        };

        let mut child = create_proc().spawn()?;

        if let Some(m) = child.stdout.take() {
            Self::transfer_to_file(
                m,
                frontend_logs_dir.join(format!("log-stdout-{}", self.build_id)),
            )
            .await?;
        }

        if let Some(m) = child.stderr.take() {
            Self::transfer_to_file(
                m,
                frontend_logs_dir.join(format!("log-stderr-{}", self.build_id)),
            )
            .await?;
        }

        let status = child.wait().await?;

        // We try again with logs printed to console.
        if !status.success() {
            if !self.is_watch_build {
                bail!("trunk failed with status {}", status);
            }

            let mut proc = create_proc();
            proc.stdout(Stdio::inherit()).stderr(Stdio::inherit());

            let mut child = proc.spawn()?;
            let status = child.wait().await?;

            if !status.success() {
                bail!("trunk failed with status {}", status);
            }
        }

        Ok(frontend_build_dir)
    }

    pub async fn build_backend(&self) -> Result<PathBuf> {
        use tokio::process::Command;

        let frontend_build_dir = self.frontend_build_dir().await?;
        let backend_logs_dir = self.paths.backend_logs_dir().await?;
        let workspace_dir = self.paths.workspace_dir().await?;
        let backend_build_dir = self.backend_build_dir().await?;

        let create_proc = || {
            let mut proc = Command::new("cargo");
            proc.arg("build")
                .arg("--bin")
                .arg(&self.manifest.dev_server.bin_name)
                .current_dir(workspace_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            if let Some(m) = self.profile.to_profile_argument() {
                proc.arg(m);
            }

            if let Some(ref m) = self.backend_target {
                proc.arg(format!("--target={}", m));
            }

            let envs = self.env_file.load(workspace_dir);
            proc.envs(envs);

            if !self.is_watch_build {
                proc.stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .env("RUSTFLAGS", "--cfg stellation_embedded_frontend");
            }

            proc.env("STELLATION_FRONTEND_BUILD_DIR", frontend_build_dir);

            proc
        };

        let mut child = create_proc().spawn()?;

        if let Some(m) = child.stdout.take() {
            Self::transfer_to_file(
                m,
                backend_logs_dir.join(format!("log-stdout-{}", self.build_id)),
            )
            .await?;
        }

        if let Some(m) = child.stderr.take() {
            Self::transfer_to_file(
                m,
                backend_logs_dir.join(format!("log-stderr-{}", self.build_id)),
            )
            .await?;
        }

        let status = child.wait().await?;

        // We try again with logs printed to console.
        if !status.success() {
            if !self.is_watch_build {
                bail!("trunk failed with status {}", status);
            }

            let mut proc = create_proc();
            proc.stdout(Stdio::inherit()).stderr(Stdio::inherit());

            let mut child = proc.spawn()?;
            let status = child.wait().await?;

            if !status.success() {
                bail!("trunk failed with status {}", status);
            }
        }

        // Copy artifact from target directory.
        let pkg_meta_output = Command::new("cargo")
            .arg("metadata")
            .arg("--format-version=1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(workspace_dir)
            .spawn()?
            .wait_with_output()
            .await
            .context("failed to read package metadata")?;

        if !pkg_meta_output.status.success() {
            bail!(
                "cargo metadata failed with status {}",
                pkg_meta_output.status
            );
        }

        let meta: Metadata = serde_json::from_slice(&pkg_meta_output.stdout)
            .context("failed to parse package metadata")?;

        let mut bin_path = meta.target_directory.into_std_path_buf();

        if let Some(ref m) = self.backend_target {
            bin_path = bin_path.join(m);
        }

        bin_path = bin_path
            .join(self.profile.name())
            .join(&self.manifest.dev_server.bin_name);

        let backend_bin_path = backend_build_dir.join(&self.manifest.dev_server.bin_name);

        fs::copy(bin_path, &backend_bin_path)
            .await
            .context("failed to copy binary")?;

        Ok(backend_bin_path)
    }
}
