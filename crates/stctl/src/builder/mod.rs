use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use cargo_metadata::Metadata;
use tokio::fs;
use tokio::sync::OnceCell;

use crate::builder::pipeline::{Pipeline, PipelineConfig};
use crate::env_file::EnvFile;
use crate::manifest::Manifest;
use crate::paths::Paths;
use crate::profile::Profile;
use crate::utils::random_str;
use crate::Stctl;

mod pipeline;

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

    pub async fn build_frontend(&self) -> Result<&Path> {
        let frontend_build_dir = self.frontend_build_dir().await?;
        let workspace_dir = self.paths.workspace_dir().await?;

        let pipeline_config = PipelineConfig::builder()
            .output_dir(frontend_build_dir)
            .public_url("/")
            .should_optimize(self.profile.should_optimize())
            .build();

        Pipeline::new(pipeline_config)
            .build(workspace_dir.join("index.html"))
            .await?;

        Ok(frontend_build_dir)
    }

    pub async fn build_backend(&self) -> Result<PathBuf> {
        use tokio::process::Command;

        let frontend_build_dir = self.frontend_build_dir().await?;
        let workspace_dir = self.paths.workspace_dir().await?;
        let backend_build_dir = self.backend_build_dir().await?;

        let create_proc = || {
            let mut proc = Command::new("cargo");
            proc.arg("build")
                .arg("--bin")
                .arg(&self.manifest.dev_server.bin_name)
                .current_dir(workspace_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
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
                proc.env("RUSTFLAGS", "--cfg stellation_embedded_frontend");
            }

            proc.env("STELLATION_FRONTEND_BUILD_DIR", frontend_build_dir);

            proc
        };

        let mut child = create_proc().spawn()?;

        let status = child.wait().await?;

        // We try again with logs printed to console.
        if !status.success() {
            bail!("cargo failed with status {}", status);
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
