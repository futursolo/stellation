//! A type to clone fn once per thread.

use std::sync::{Arc, Mutex};

use thread_local::ThreadLocal;

// type BoxedSendFn<IN, OUT> = Box<dyn Send + Fn(IN) -> OUT>;

// pub(crate) struct SendFn<IN, OUT> {
//     inner: Arc<ThreadLocal<BoxedSendFn<IN, OUT>>>,
//     create_inner: Arc<dyn Send + Sync + Fn() -> BoxedSendFn<IN, OUT>>,
// }

// impl<IN, OUT> Clone for SendFn<IN, OUT> {
//     fn clone(&self) -> Self {
//         Self {
//             inner: self.inner.clone(),
//             create_inner: self.create_inner.clone(),
//         }
//     }
// }

// impl<IN, OUT> SendFn<IN, OUT> {
//     pub fn new<F>(f: F) -> Self
//     where
//         F: 'static + Clone + Send + Fn(IN) -> OUT,
//     {
//         let clonable_inner = Arc::new(Mutex::new(f));
//         let create_inner = move || -> BoxedSendFn<IN, OUT> {
//             let clonable_inner = clonable_inner.lock().expect("failed to lock?");
//             Box::new(clonable_inner.clone())
//         };

//         Self {
//             inner: Arc::new(ThreadLocal::new()),
//             create_inner: Arc::new(create_inner),
//         }
//     }

//     pub fn emit(&self, input: IN) -> OUT {
//         let inner = self.inner.get_or(|| (self.create_inner)());
//         inner(input)
//     }
// }

type BoxedUnitSendFn<OUT> = Box<dyn Send + Fn() -> OUT>;

pub(crate) struct UnitSendFn<OUT> {
    inner: Arc<ThreadLocal<BoxedUnitSendFn<OUT>>>,
    create_inner: Arc<dyn Send + Sync + Fn() -> BoxedUnitSendFn<OUT>>,
}

impl<OUT> Clone for UnitSendFn<OUT> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            create_inner: self.create_inner.clone(),
        }
    }
}

impl<OUT> UnitSendFn<OUT> {
    pub fn new<F>(f: F) -> Self
    where
        F: 'static + Clone + Send + Fn() -> OUT,
    {
        let clonable_inner = Arc::new(Mutex::new(f));
        let create_inner = move || -> BoxedUnitSendFn<OUT> {
            let clonable_inner = clonable_inner.lock().expect("failed to lock?");
            Box::new(clonable_inner.clone())
        };

        Self {
            inner: Arc::new(ThreadLocal::new()),
            create_inner: Arc::new(create_inner),
        }
    }

    pub fn emit(&self) -> OUT {
        let inner = self.inner.get_or(|| (self.create_inner)());
        inner()
    }
}
