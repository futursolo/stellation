use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use trunk_pipelines::assets::{
    Asset, CopyDir, CopyDirConfig, CopyFile, CopyFileConfig, Css, CssConfig, Icon, IconConfig,
    Inline, Js, JsConfig, RustApp, RustAppConfig, Sass, SassConfig, TailwindCss, TailwindCssConfig,
};
use trunk_pipelines::html::{HtmlPipeline, HtmlPipelineConfig};
use trunk_util::Features;
use typed_builder::TypedBuilder;

static ASSET_ATTR: &str = "data-st";

#[derive(Debug, Clone, TypedBuilder)]
pub(crate) struct PipelineConfig {
    #[builder(setter(into))]
    output_dir: PathBuf,
    #[builder(default = "/".into(), setter(into))]
    public_url: Cow<'static, str>,

    should_optimize: bool,

    #[builder(default)]
    sass_version: Option<String>,
    #[builder(default)]
    tailwind_css_version: Option<String>,
    #[builder(default)]
    wasm_opt_version: Option<String>,
    #[builder(default)]
    wasm_bindgen_version: Option<String>,

    #[builder(default)]
    loader_script: Option<Cow<'static, str>>,
}

impl CssConfig for PipelineConfig {
    fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    fn public_url(&self) -> &str {
        &self.public_url
    }

    fn should_hash(&self) -> bool {
        true
    }
}

impl SassConfig for PipelineConfig {
    fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    fn should_hash(&self) -> bool {
        true
    }

    fn public_url(&self) -> &str {
        &self.public_url
    }

    fn should_optimize(&self) -> bool {
        self.should_optimize
    }

    fn version(&self) -> Option<&str> {
        self.sass_version.as_deref()
    }
}

impl JsConfig for PipelineConfig {
    fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    fn public_url(&self) -> &str {
        &self.public_url
    }

    fn should_hash(&self) -> bool {
        true
    }

    fn asset_attr(&self) -> &str {
        ASSET_ATTR
    }
}

impl IconConfig for PipelineConfig {
    fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    fn public_url(&self) -> &str {
        &self.public_url
    }

    fn should_hash(&self) -> bool {
        true
    }
}

impl TailwindCssConfig for PipelineConfig {
    fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    fn public_url(&self) -> &str {
        &self.public_url
    }

    fn should_hash(&self) -> bool {
        true
    }

    fn should_optimize(&self) -> bool {
        self.should_optimize
    }

    fn version(&self) -> Option<&str> {
        self.tailwind_css_version.as_deref()
    }
}

impl RustAppConfig for PipelineConfig {
    fn allow_concurrent_cargo_build(&self) -> bool {
        false
    }

    fn public_url(&self) -> &str {
        &self.public_url
    }

    fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    fn wasm_opt_version(&self) -> Option<&str> {
        self.wasm_opt_version.as_deref()
    }

    fn wasm_bindgen_version(&self) -> Option<&str> {
        self.wasm_bindgen_version.as_deref()
    }

    fn should_optimize(&self) -> bool {
        self.should_optimize
    }

    fn should_hash(&self) -> bool {
        true
    }

    fn format_script(&self, _script_path: &str, _wasm_path: &str) -> Option<String> {
        None
    }

    fn format_preload(&self, _script_path: &str, _wasm_path: &str) -> Option<String> {
        None
    }

    fn cargo_features(&self) -> Option<&Features> {
        None
    }
}

impl CopyFileConfig for PipelineConfig {
    fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

impl CopyDirConfig for PipelineConfig {
    fn output_dir(&self) -> &Path {
        &self.output_dir
    }
}

impl HtmlPipelineConfig for PipelineConfig {
    fn spawn_build_hooks(
        self: &Arc<Self>,
    ) -> Option<tokio::task::JoinHandle<trunk_util::Result<()>>> {
        None
    }

    fn spawn_post_build_hooks(
        self: &Arc<Self>,
    ) -> Option<tokio::task::JoinHandle<trunk_util::Result<()>>> {
        None
    }

    fn spawn_pre_build_hooks(
        self: &Arc<Self>,
    ) -> Option<tokio::task::JoinHandle<trunk_util::Result<()>>> {
        None
    }

    fn append_body_str(&self) -> Option<Cow<'_, str>> {
        self.loader_script.as_deref().map(Cow::from)
    }

    fn asset_attr(&self) -> &str {
        ASSET_ATTR
    }
}

pub(crate) struct Pipeline {
    config: Arc<PipelineConfig>,
}

impl Pipeline {
    pub fn new<C>(config: C) -> Self
    where
        C: Into<Arc<PipelineConfig>>,
    {
        let config = config.into();

        Self { config }
    }

    pub async fn build<P>(self, path: P) -> Result<()>
    where
        P: Into<PathBuf>,
    {
        // TODO: make backend compile into a pipeline as well.
        let asset_pipeline = RustApp::new(self.config.clone())
            .chain(CopyDir::new(self.config.clone()))
            .chain(CopyFile::new(self.config.clone()))
            .chain(Css::new(self.config.clone()))
            .chain(Icon::new(self.config.clone()))
            .chain(Inline::new())
            .chain(Js::new(self.config.clone()))
            .chain(Sass::new(self.config.clone()))
            .chain(TailwindCss::new(self.config.clone()));

        HtmlPipeline::new(path, self.config.clone(), asset_pipeline)?
            .spawn_threaded()
            .await
            .context("error joining HTML pipeline")?
            .context("error from HTML pipeline")?;

        Ok(())
    }
}
