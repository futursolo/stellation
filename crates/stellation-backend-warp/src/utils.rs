//! Warp utilities.

use futures::Future;
use yew::platform::{LocalHandle, Runtime};

/// Creates a random string.
pub(crate) fn random_str() -> String {
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

pub(crate) fn spawn_pinned_or_local<F, Fut>(create_task: F)
where
    F: FnOnce() -> Fut,
    F: Send + 'static,
    Fut: Future<Output = ()> + 'static,
{
    // We spawn into a local runtime early for higher efficiency.
    match LocalHandle::try_current() {
        Some(handle) => handle.spawn_local(create_task()),
        // TODO: Allow Overriding Runtime with Endpoint.
        None => Runtime::default().spawn_pinned(create_task),
    }
}
