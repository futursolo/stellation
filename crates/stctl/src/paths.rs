use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::fs;
use tokio::sync::OnceCell;

#[derive(Debug, Clone)]
pub(crate) struct Paths {
    workspace_dir: PathBuf,
    build_dir: OnceCell<PathBuf>,
    data_dir: OnceCell<PathBuf>,
    frontend_data_dir: OnceCell<PathBuf>,
    backend_data_dir: OnceCell<PathBuf>,

    frontend_builds_dir: OnceCell<PathBuf>,
    backend_builds_dir: OnceCell<PathBuf>,

    frontend_logs_dir: OnceCell<PathBuf>,
    backend_logs_dir: OnceCell<PathBuf>,
}

impl Paths {
    /// Creates a new `Paths` for stctl.
    pub async fn new<P>(manifest_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let manifest_path = manifest_path.as_ref();

        let workspace_dir = manifest_path
            .canonicalize()?
            .parent()
            .context("failed to find workspace directory")
            .map(|m| m.to_owned())?;

        Ok(Self {
            workspace_dir,
            build_dir: OnceCell::new(),
            data_dir: OnceCell::new(),
            frontend_data_dir: OnceCell::new(),
            backend_data_dir: OnceCell::new(),
            frontend_builds_dir: OnceCell::new(),
            backend_builds_dir: OnceCell::new(),
            frontend_logs_dir: OnceCell::new(),
            backend_logs_dir: OnceCell::new(),
        })
    }

    /// Returns the workspace directory.
    ///
    /// This is the parent directory of `stellation.toml`.
    ///
    /// # Note
    ///
    /// This can be different than the cargo workspace directory.
    ///
    /// This determines the location of `.stellation` data directory and `build` final artifact
    /// directory. This is subject to change in future releases.
    pub async fn workspace_dir(&self) -> Result<&Path> {
        Ok(&self.workspace_dir)
    }

    /// Creates and returns the path of the build directory.
    ///
    /// This is the `build` directory in the same parent directory as `stellation.toml`.
    ///
    /// # Note
    ///
    /// This should not be confused with the `builds` directory.
    pub async fn build_dir(&self) -> Result<&Path> {
        self.build_dir
            .get_or_try_init(|| async {
                let dir = self.workspace_dir().await?.join("build");

                fs::create_dir_all(&dir)
                    .await
                    .context("failed to create build directory")?;

                Ok(dir)
            })
            .await
            .map(|m| m.as_ref())
    }

    /// Creates and returns the path of the data directory.
    ///
    /// This is the `.stellation` directory in the same parent directory as `stellation.toml`.
    pub async fn data_dir(&self) -> Result<&Path> {
        self.data_dir
            .get_or_try_init(|| async {
                let dir = self.workspace_dir().await?.join(".stellation");

                fs::create_dir_all(&dir)
                    .await
                    .context("failed to create data directory")?;

                Ok(dir)
            })
            .await
            .map(|m| m.as_ref())
    }

    /// Creates and returns the path of the frontend data directory.
    ///
    /// This is the `.stellation/frontend` directory in the same parent directory as
    /// `stellation.toml`.
    pub async fn frontend_data_dir(&self) -> Result<&Path> {
        self.frontend_data_dir
            .get_or_try_init(|| async {
                let dir = self.data_dir().await?.join("frontend");

                fs::create_dir_all(&dir)
                    .await
                    .context("failed to create frontend data directory")?;

                Ok(dir)
            })
            .await
            .map(|m| m.as_ref())
    }

    /// Creates and returns the path of the backend data directory.
    ///
    /// This is the `.stellation/backend` directory in the same parent directory as
    /// `stellation.toml`.
    pub async fn backend_data_dir(&self) -> Result<&Path> {
        self.backend_data_dir
            .get_or_try_init(|| async {
                let dir = self.data_dir().await?.join("backend");

                fs::create_dir_all(&dir)
                    .await
                    .context("failed to create backend data directory")?;

                Ok(dir)
            })
            .await
            .map(|m| m.as_ref())
    }

    /// Creates and returns the path of the frontend builds directory.
    ///
    /// This is the `.stellation/frontend/builds` directory in the same parent directory as
    /// `stellation.toml`.
    pub async fn frontend_builds_dir(&self) -> Result<&Path> {
        self.frontend_builds_dir
            .get_or_try_init(|| async {
                let dir = self.frontend_data_dir().await?.join("builds");

                fs::create_dir_all(&dir)
                    .await
                    .context("failed to create builds directory for frontend build.")?;

                Ok(dir)
            })
            .await
            .map(|m| m.as_ref())
    }

    /// Creates and returns the path of the backend builds directory.
    ///
    /// This is the `.stellation/backend/builds` directory in the same parent directory as
    /// `stellation.toml`.
    pub async fn backend_builds_dir(&self) -> Result<&Path> {
        self.backend_builds_dir
            .get_or_try_init(|| async {
                let dir = self.backend_data_dir().await?.join("builds");

                fs::create_dir_all(&dir)
                    .await
                    .context("failed to create builds directory for backend build.")?;

                Ok(dir)
            })
            .await
            .map(|m| m.as_ref())
    }

    /// Creates and returns the path of the frontend logs directory.
    ///
    /// This is the `.stellation/frontend/logs` directory in the same parent directory as
    /// `stellation.toml`.
    pub async fn frontend_logs_dir(&self) -> Result<&Path> {
        self.frontend_logs_dir
            .get_or_try_init(|| async {
                let dir = self.frontend_data_dir().await?.join("logs");

                fs::create_dir_all(&dir)
                    .await
                    .context("failed to create logs directory for frontend build.")?;

                Ok(dir)
            })
            .await
            .map(|m| m.as_ref())
    }

    /// Creates and returns the path of the backend logs directory.
    ///
    /// This is the `.stellation/backend/logs` directory in the same parent directory as
    /// `stellation.toml`.
    pub async fn backend_logs_dir(&self) -> Result<&Path> {
        self.backend_logs_dir
            .get_or_try_init(|| async {
                let dir = self.backend_data_dir().await?.join("logs");

                fs::create_dir_all(&dir)
                    .await
                    .context("failed to create logs directory for backend build.")?;

                Ok(dir)
            })
            .await
            .map(|m| m.as_ref())
    }
}
