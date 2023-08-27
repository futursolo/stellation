use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct EnvFile {
    name: String,
}

impl EnvFile {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }

    pub fn load<P>(&self, workspace_dir: P) -> HashMap<String, String>
    where
        P: AsRef<Path>,
    {
        let workspace_dir = workspace_dir.as_ref();
        let mut envs = HashMap::new();

        for env_name in [
            Cow::from(".env"),
            ".env.local".into(),
            format!(".env.{}", self.name).into(),
            format!(".env.{}.local", self.name).into(),
        ] {
            let path = workspace_dir.join(env_name.as_ref());
            if path.exists() {
                if let Err(e) = dotenvy::from_path_iter(&path).and_then(|m| {
                    for i in m {
                        let (k, v) = i?;
                        if env::var(&k).is_ok() {
                            // environment variables inherited from current process have a higher
                            // priority.
                            continue;
                        }

                        envs.insert(k, v);
                    }

                    Ok(())
                }) {
                    tracing::warn!(path = %path.display(), reason = ?e, "failed to load environment file");
                }
            }
        }

        envs
    }
}
