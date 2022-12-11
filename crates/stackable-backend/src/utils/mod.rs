mod thread_local;

use rand::distributions::Alphanumeric;
use rand::Rng;

pub use self::thread_local::ThreadLocalLazy;

#[cfg(feature = "warp-filter")]
pub(crate) fn random_str() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}
