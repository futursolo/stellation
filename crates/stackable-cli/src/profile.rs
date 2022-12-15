#[derive(Debug)]
pub(crate) struct Profile {
    name: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self::new_debug()
    }
}

impl Profile {
    pub fn new_debug() -> Self {
        Self {
            name: "debug".to_string(),
        }
    }

    pub fn new_release() -> Self {
        Self {
            name: "release".to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn to_profile_argument(&self) -> Option<String> {
        match self.name() {
            "debug" => None,
            "release" => Some("--release".to_string()),
            other => Some(format!("--profile={}", other)),
        }
    }
}
