use std::time::SystemTime;

use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub(crate) fn random_str() -> Result<String> {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    Ok(format!(
        "{}-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
        s
    ))
}
