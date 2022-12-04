use core::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use yew::prelude::*;

pub struct Endpoint<COMP, F>
where
    COMP: BaseComponent,
{
    create_props: F,
    _marker: PhantomData<COMP>,
}

impl<COMP, F> fmt::Debug for Endpoint<COMP, F>
where
    COMP: BaseComponent,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Endpoint<_>")
    }
}

impl<COMP, F> Endpoint<COMP, F>
where
    COMP: BaseComponent,
{
    pub fn new() -> Endpoint<COMP, impl 'static + Clone + Send + Fn() -> COMP::Properties>
    where
        COMP::Properties: Default,
    {
        Endpoint::<COMP, _>::with_props(COMP::Properties::default)
    }

    pub fn with_props(f: F) -> Self
    where
        F: 'static + Clone + Send + Fn() -> COMP::Properties,
    {
        Self {
            create_props: f,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "tower-service")]
mod feat_tower_service {
    use std::convert::Infallible;
    use std::future::Future;

    use futures::channel::oneshot as sync_oneshot;
    use hyper::{Body, Request, Response};
    use tower::{Service, ServiceExt};
    use warp::Filter;
    use yew::platform::{LocalHandle, Runtime};

    use super::*;
    impl<COMP, F> Endpoint<COMP, F>
    where
        COMP: BaseComponent,
        F: 'static + Clone + Send + Fn() -> COMP::Properties,
    {
        async fn render_html_inner(
            index_html_s: Arc<str>,
            create_props: F,
            tx: sync_oneshot::Sender<String>,
        ) where
            F: 'static + Clone + Send + Fn() -> COMP::Properties,
        {
            let props = create_props();
            let body_s = yew::LocalServerRenderer::<COMP>::with_props(props)
                .render()
                .await;

            let s = index_html_s.replace("<!--%STACKABLE_BODY%-->", &body_s);

            let _ = tx.send(s);
        }

        async fn render_html(index_html_s: Arc<str>, create_props: F) -> String {
            let (tx, rx) = sync_oneshot::channel();

            let create_render_inner = move || async move {
                Self::render_html_inner(index_html_s, create_props, tx).await;
            };

            // We spawn into a local runtime early for higher efficiency.
            match LocalHandle::try_current() {
                Some(handle) => handle.spawn_local(create_render_inner()),
                // TODO: Allow Overriding Runtime with Endpoint.
                None => Runtime::default().spawn_pinned(create_render_inner),
            }

            rx.await.expect("renderer panicked?")
        }

        pub fn into_tower_service(
            self,
        ) -> impl 'static
               + Clone
               + Service<
            Request<Body>,
            Response = Response<Body>,
            Error = Infallible,
            Future = impl 'static + Send + Future<Output = Result<Response<Body>, Infallible>>,
        > {
            let Self { create_props, .. } = self;

            let index_html_s: Arc<str> = Arc::from("");

            let index_html_f = warp::get().then(move || {
                let index_html_s = index_html_s.clone();
                let create_props = create_props.clone();

                Self::render_html(index_html_s, create_props)
            });

            let routes = index_html_f;

            warp::service(routes).boxed_clone()
        }
    }
}
