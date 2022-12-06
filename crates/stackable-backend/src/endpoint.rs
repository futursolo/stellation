use core::fmt;
use std::sync::Arc;

use yew::prelude::*;

use crate::dev_env::DevEnv;
use crate::utils::send_fn::UnitSendFn;

pub struct Endpoint<COMP>
where
    COMP: BaseComponent,
{
    create_props: UnitSendFn<COMP::Properties>,
    #[cfg(feature = "tower-service")]
    dev_env: Option<DevEnv>,
}

impl<COMP> fmt::Debug for Endpoint<COMP>
where
    COMP: BaseComponent,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Endpoint<_>")
    }
}

impl<COMP> Default for Endpoint<COMP>
where
    COMP: BaseComponent,
    COMP::Properties: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<COMP> Endpoint<COMP>
where
    COMP: BaseComponent,
{
    pub fn new() -> Endpoint<COMP>
    where
        COMP::Properties: Default,
    {
        Endpoint::<COMP>::with_props(COMP::Properties::default)
    }

    pub fn with_props<F>(f: F) -> Self
    where
        F: 'static + Clone + Send + Fn() -> COMP::Properties,
    {
        Self {
            create_props: UnitSendFn::new(f),
            #[cfg(feature = "tower-service")]
            dev_env: None,
        }
    }
}

#[cfg(feature = "warp-filter")]
mod feat_warp_filter {
    use std::collections::HashMap;
    use std::future::Future;
    use std::path::Path;

    use bounce::helmet::render_static;
    use futures::channel::oneshot as sync_oneshot;
    use tokio::fs;
    use warp::path::FullPath;
    use warp::{Filter, Rejection, Reply};
    use yew::platform::{LocalHandle, Runtime};

    use super::*;
    use crate::root::{StackableRoot, StackableRootProps};
    impl<COMP> Endpoint<COMP>
    where
        COMP: BaseComponent,
    {
        async fn render_html_inner(
            index_html_path: Arc<Path>,
            path: String,
            queries: HashMap<String, String>,
            create_props: UnitSendFn<COMP::Properties>,
            tx: sync_oneshot::Sender<String>,
        ) {
            let props = create_props.emit();
            let children = html! {
                <COMP ..props />
            };

            let (reader, writer) = render_static();

            let body_s =
                yew::LocalServerRenderer::<StackableRoot>::with_props(StackableRootProps {
                    children,
                    helmet_writer: writer,
                    path,
                    queries,
                })
                .render()
                .await;

            let mut head_s = String::new();
            let helmet_tags = reader.render().await;

            for tag in helmet_tags {
                let _ = tag.write_static(&mut head_s);
            }

            // With development server, we read index.html every time.
            let index_html_s = fs::read_to_string(&index_html_path)
                .await
                .expect("TODO: implement failure.");

            let s = index_html_s
                .replace("<!--%STACKABLE_HEAD%-->", &head_s)
                .replace("<!--%STACKABLE_BODY%-->", &body_s);

            let _ = tx.send(s);
        }

        async fn render_html(
            index_html_path: Arc<Path>,
            path: String,
            queries: HashMap<String, String>,
            create_props: UnitSendFn<COMP::Properties>,
        ) -> impl Reply {
            let (tx, rx) = sync_oneshot::channel();

            let create_render_inner = move || async move {
                Self::render_html_inner(index_html_path, path, queries, create_props, tx).await;
            };

            // We spawn into a local runtime early for higher efficiency.
            match LocalHandle::try_current() {
                Some(handle) => handle.spawn_local(create_render_inner()),
                // TODO: Allow Overriding Runtime with Endpoint.
                None => Runtime::default().spawn_pinned(create_render_inner),
            }

            warp::reply::html(rx.await.expect("renderer panicked?"))
        }

        pub fn set_dev_env(&mut self, e: DevEnv) {
            self.dev_env = Some(e);
        }

        pub fn into_warp_filter(
            self,
        ) -> impl Clone
               + Send
               + Filter<
            Extract = impl Reply,
            Error = Rejection,
            Future = impl Future<Output = Result<impl Reply, Rejection>>,
        > {
            let Self { create_props, .. } = self;
            let dev_server_build_path = self
                .dev_env
                .expect("running without development server is not implemented")
                .dev_server_build_path;
            let index_html_path: Arc<Path> = Arc::from(dev_server_build_path.join("index.html"));

            let index_html_f = warp::get().and(warp::path::full()).and(warp::query()).then(
                move |path: FullPath, queries: HashMap<String, String>| {
                    let index_html_path = index_html_path.clone();
                    let create_props = create_props.clone();
                    let path = path.as_str().to_string();

                    Self::render_html(index_html_path, path, queries, create_props)
                },
            );

            warp::path::end()
                .and(index_html_f.clone())
                .or(warp::fs::dir(dev_server_build_path))
                .or(index_html_f)
                .with(warp::trace::request())
        }
    }
}

#[cfg(feature = "tower-service")]
mod feat_tower_service {
    use std::convert::Infallible;
    use std::future::Future;

    use hyper::{Body, Request, Response};
    use tower::Service;

    use super::*;
    impl<COMP> Endpoint<COMP>
    where
        COMP: BaseComponent,
    {
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
            let routes = self.into_warp_filter();
            warp::service(routes)
        }
    }
}
