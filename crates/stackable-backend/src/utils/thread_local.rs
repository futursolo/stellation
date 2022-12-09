//! A type to clone fn once per thread.

use std::fmt;
use std::sync::{Arc, Mutex};

use thread_local::ThreadLocal;

pub struct ThreadLocalLazy<T: Send> {
    value: Arc<ThreadLocal<T>>,
    create_value: Arc<dyn Send + Sync + Fn() -> T>,
}

impl<T: Send> fmt::Debug for ThreadLocalLazy<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ThreadLocalLazy<_>")
    }
}

impl<T> Clone for ThreadLocalLazy<T>
where
    T: 'static + Send,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            create_value: self.create_value.clone(),
        }
    }
}

impl<T> ThreadLocalLazy<T>
where
    T: 'static + Send,
{
    pub fn new<F>(f: F) -> Self
    where
        F: 'static + Send + Fn() -> T,
    {
        let clonable_inner = Arc::new(Mutex::new(f));
        let create_inner = move || -> T {
            let clonable_inner = clonable_inner.lock().expect("failed to lock?");
            clonable_inner()
        };

        Self {
            value: Arc::new(ThreadLocal::new()),
            create_value: Arc::new(create_inner),
        }
    }

    pub fn get(&self) -> &T {
        self.value.get_or(|| (self.create_value)())
    }
}
