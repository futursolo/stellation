use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct DevEnv {
    pub listen_addr: String,
    pub dev_server_build_path: PathBuf,
}

impl DevEnv {
    pub fn from_env() -> Option<Self> {
        Some(DevEnv {
            listen_addr: env::var("STACKCTL_LISTEN_ADDR").ok()?,
            dev_server_build_path: env::var_os("STACKCTL_DEV_SERVER_BUILD_PATH")?.into(),
        })
    }
}
