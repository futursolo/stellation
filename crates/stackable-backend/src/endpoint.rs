use core::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use stackable_bridge::Bridge;
use yew::prelude::*;

use crate::dev_env::DevEnv;
use crate::utils::thread_local::ThreadLocalLazy;
use crate::ServerAppProps;

type BoxedSendFn<IN, OUT> = Box<dyn Send + Fn(IN) -> OUT>;

type SendFn<IN, OUT> = ThreadLocalLazy<BoxedSendFn<IN, OUT>>;

pub struct Endpoint<COMP, CTX = ()>
where
    COMP: BaseComponent,
{
    affix_context: SendFn<ServerAppProps<()>, ServerAppProps<CTX>>,
    bridge: Option<Bridge>,
    _marker: PhantomData<COMP>,
    #[cfg(feature = "tower-service")]
    dev_env: Option<DevEnv>,
}

impl<COMP, CTX> fmt::Debug for Endpoint<COMP, CTX>
where
    COMP: BaseComponent,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Endpoint<_>")
    }
}

impl<COMP, CTX> Default for Endpoint<COMP, CTX>
where
    COMP: BaseComponent,
    CTX: 'static + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<COMP, CTX> Endpoint<COMP, CTX>
where
    COMP: BaseComponent,
    CTX: 'static,
{
    pub fn new() -> Self
    where
        CTX: Default,
    {
        Endpoint::<COMP, CTX>::with_create_context(|m| m.with_context(CTX::default()))
    }

    pub fn with_create_context<F>(create_context: F) -> Self
    where
        F: 'static + Clone + Send + Fn(ServerAppProps<()>) -> ServerAppProps<CTX>,
    {
        Self {
            affix_context: SendFn::<ServerAppProps<()>, ServerAppProps<CTX>>::new(move || {
                Box::new(create_context.clone())
            }),
            bridge: None,
            #[cfg(feature = "tower-service")]
            dev_env: None,
            _marker: PhantomData,
        }
    }

    pub fn with_bridge(mut self, bridge: Bridge) -> Self {
        self.bridge = Some(bridge);
        self
    }
}

#[cfg(feature = "warp-filter")]
mod feat_warp_filter {
    use std::future::Future;
    use std::path::Path;

    use bounce::helmet::render_static;
    use futures::channel::oneshot as sync_oneshot;
    use hyper::body::Bytes;
    use tokio::fs;
    use warp::body::bytes;
    use warp::fs::File;
    use warp::path::FullPath;
    use warp::{header, reply, Filter, Rejection, Reply};
    use yew::platform::{LocalHandle, Runtime};

    use super::*;
    use crate::root::{StackableRoot, StackableRootProps};
    impl<COMP, CTX> Endpoint<COMP, CTX>
    where
        COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
        CTX: 'static,
    {
        async fn render_html_inner(
            index_html_path: Arc<Path>,
            props: ServerAppProps<()>,
            affix_context: SendFn<ServerAppProps<()>, ServerAppProps<CTX>>,
            tx: sync_oneshot::Sender<String>,
        ) {
            let props = (affix_context.get())(props);
            let (reader, writer) = render_static();

            let body_s = yew::LocalServerRenderer::<StackableRoot<COMP, CTX>>::with_props(
                StackableRootProps {
                    server_app_props: props,
                    helmet_writer: writer,
                    _marker: PhantomData,
                },
            )
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
            props: ServerAppProps<()>,
            affix_context: SendFn<ServerAppProps<()>, ServerAppProps<CTX>>,
        ) -> impl Reply {
            let (tx, rx) = sync_oneshot::channel();

            let create_render_inner = move || async move {
                Self::render_html_inner(index_html_path, props, affix_context, tx).await;
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
            let Self {
                affix_context,
                bridge,
                ..
            } = self;
            let dev_server_build_path = self
                .dev_env
                .expect("running without development server is not implemented")
                .dev_server_build_path;
            let index_html_path: Arc<Path> = Arc::from(dev_server_build_path.join("index.html"));

            let index_html_f = warp::get()
                .and(warp::path::full())
                .and(
                    warp::query::raw()
                        .or_else(|_| async move { Ok::<_, Rejection>((String::new(),)) }),
                )
                .then(move |path: FullPath, raw_queries| {
                    let index_html_path = index_html_path.clone();
                    let affix_context = affix_context.clone();
                    let props = ServerAppProps::from_warp_request(path, raw_queries);

                    async move {
                        Self::render_html(index_html_path, props, affix_context)
                            .await
                            .into_response()
                    }
                });

            let mut routes = warp::path::end()
                .and(index_html_f.clone())
                .or(warp::fs::dir(dev_server_build_path)
                    .then(|m: File| async move { m.into_response() })
                    .boxed())
                .unify()
                .or(index_html_f)
                .unify()
                .boxed();

            if let Some(m) = bridge {
                let bridge = Arc::new(m);

                let bridge_f = warp::post()
                    .and(warp::path::path("_bridge"))
                    .and(header("X-Bridge-Type-Idx"))
                    .and(header::exact_ignore_case(
                        "content-type",
                        "application/x-bincode",
                    ))
                    .and(bytes())
                    .then(move |index: usize, input: Bytes| {
                        let bridge = bridge.clone();
                        let (tx, rx) = sync_oneshot::channel();

                        let resolve_encoded = move || async move {
                            let output = bridge.resolve_encoded(index, &input).await;
                            let _ = tx.send(output);
                        };

                        match LocalHandle::try_current() {
                            Some(handle) => handle.spawn_local(resolve_encoded()),
                            // TODO: Allow Overriding Runtime with Endpoint.
                            None => Runtime::default().spawn_pinned(resolve_encoded),
                        }

                        async move {
                            reply::with_header(
                                rx.await.expect("didn't receive result?"),
                                "content-type",
                                "application/x-bincode",
                            )
                            .into_response()
                        }
                    });

                routes = routes.or(bridge_f).unify().boxed();
            }

            routes.with(warp::trace::request())
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
    impl<COMP, CTX> Endpoint<COMP, CTX>
    where
        COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
        CTX: 'static,
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
