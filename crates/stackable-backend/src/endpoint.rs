use core::fmt;
use std::marker::PhantomData;

use stackable_bridge::Bridge;
use yew::prelude::*;

use crate::props::ServerAppProps;
use crate::utils::ThreadLocalLazy;

type BoxedSendFn<IN, OUT> = Box<dyn Send + Fn(IN) -> OUT>;

type SendFn<IN, OUT> = ThreadLocalLazy<BoxedSendFn<IN, OUT>>;

pub struct Endpoint<COMP, CTX = ()>
where
    COMP: BaseComponent,
{
    #[allow(dead_code)]
    affix_context: SendFn<ServerAppProps<()>, ServerAppProps<CTX>>,
    bridge: Option<Bridge>,
    #[cfg(feature = "warp-filter")]
    frontend: Option<crate::Frontend>,

    _marker: PhantomData<COMP>,
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
            #[cfg(feature = "warp-filter")]
            frontend: None,
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
    use std::borrow::Cow;
    use std::fmt::Write;
    use std::future::Future;

    use bounce::helmet::render_static;
    use bytes::Bytes;
    use futures::channel::oneshot as sync_oneshot;
    use http::status::StatusCode;
    use stackable_bridge::BridgeError;
    use tokio::fs;
    use warp::body::bytes;
    use warp::path::FullPath;
    use warp::reject::not_found;
    use warp::reply::Response;
    use warp::{header, log, reply, Filter, Rejection, Reply};
    use yew::platform::{LocalHandle, Runtime};

    use super::*;
    use crate::frontend::IndexHtml;
    use crate::root::{StackableRoot, StackableRootProps};
    use crate::Frontend;

    impl<COMP, CTX> Endpoint<COMP, CTX>
    where
        COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
        CTX: 'static,
    {
        async fn render_html_inner(
            index_html: IndexHtml,
            props: ServerAppProps<()>,
            bridge: Bridge,
            affix_context: SendFn<ServerAppProps<()>, ServerAppProps<CTX>>,
            tx: sync_oneshot::Sender<String>,
        ) {
            let props = (affix_context.get())(props);
            let (reader, writer) = render_static();

            let body_s = yew::LocalServerRenderer::<StackableRoot<COMP, CTX>>::with_props(
                StackableRootProps {
                    server_app_props: props,
                    helmet_writer: writer,
                    bridge,
                },
            )
            .render()
            .await;

            let mut head_s = String::new();
            let helmet_tags = reader.render().await;

            for tag in helmet_tags {
                let _ = tag.write_static(&mut head_s);
            }
            let _ = write!(
                &mut head_s,
                r#"<meta name="stackable-mode" content="hydrate">"#
            );

            // With development server, we read index.html every time.
            let index_html_s = match index_html {
                IndexHtml::Path(p) => fs::read_to_string(&p)
                    .await
                    .map(Cow::from)
                    .expect("TODO: implement failure."),

                IndexHtml::Embedded(ref s) => s.as_ref().into(),
            };

            let s = index_html_s
                .replace("<!--%STACKABLE_HEAD%-->", &head_s)
                .replace("<!--%STACKABLE_BODY%-->", &body_s);

            let _ = tx.send(s);
        }

        async fn render_html(
            index_html: IndexHtml,
            props: ServerAppProps<()>,
            bridge: Bridge,
            affix_context: SendFn<ServerAppProps<()>, ServerAppProps<CTX>>,
        ) -> impl Reply {
            let (tx, rx) = sync_oneshot::channel();

            let create_render_inner = move || async move {
                Self::render_html_inner(index_html, props, bridge, affix_context, tx).await;
            };

            // We spawn into a local runtime early for higher efficiency.
            match LocalHandle::try_current() {
                Some(handle) => handle.spawn_local(create_render_inner()),
                // TODO: Allow Overriding Runtime with Endpoint.
                None => Runtime::default().spawn_pinned(create_render_inner),
            }

            warp::reply::html(rx.await.expect("renderer panicked?"))
        }

        fn create_index_filter(
            index_html: IndexHtml,
            bridge: Option<Bridge>,
            affix_context: SendFn<ServerAppProps<()>, ServerAppProps<CTX>>,
        ) -> impl Clone
               + Send
               + Filter<
            Extract = (Response,),
            Error = Rejection,
            Future = impl Future<Output = Result<(Response,), Rejection>>,
        > {
            warp::get()
                .and(warp::path::full())
                .and(
                    warp::query::raw()
                        .or_else(|_| async move { Ok::<_, Rejection>((String::new(),)) }),
                )
                .then(move |path: FullPath, raw_queries| {
                    let index_html = index_html.clone();
                    let affix_context = affix_context.clone();
                    let props = ServerAppProps::from_warp_request(path, raw_queries);
                    let bridge = bridge.clone();

                    async move {
                        Self::render_html(
                            index_html,
                            props,
                            bridge.unwrap_or_default(),
                            affix_context,
                        )
                        .await
                        .into_response()
                    }
                })
        }

        fn create_bridge_filter(
            bridge: Bridge,
        ) -> impl Clone
               + Send
               + Filter<
            Extract = (Response,),
            Error = Rejection,
            Future = impl Future<Output = Result<(Response,), Rejection>>,
        > {
            warp::path::path("_bridge")
                .and(warp::post())
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
                        let content = rx.await.expect("didn't receive result?");

                        match content {
                            Ok(m) => reply::with_header(m, "content-type", "application/x-bincode")
                                .into_response(),
                            Err(BridgeError::Encoding(_))
                            | Err(BridgeError::InvalidIndex(_))
                            | Err(BridgeError::InvalidType(_)) => {
                                reply::with_status("", StatusCode::BAD_REQUEST).into_response()
                            }
                            Err(BridgeError::Network(_)) => {
                                reply::with_status("", StatusCode::INTERNAL_SERVER_ERROR)
                                    .into_response()
                            }
                        }
                    }
                })
        }

        pub fn with_frontend(mut self, frontend: Frontend) -> Self {
            self.frontend = Some(frontend);

            self
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
                frontend,
                ..
            } = self;
            let index_html_f = frontend
                .clone()
                .map(|m| Self::create_index_filter(m.index_html(), bridge.clone(), affix_context));

            let bridge_f = bridge.map(Self::create_bridge_filter);

            let mut routes = match index_html_f.clone() {
                None => warp::path::end()
                    .and_then(|| async move { Err::<Response, Rejection>(not_found()) })
                    .boxed(),
                Some(m) => warp::path::end().and(m).boxed(),
            };

            if let Some(m) = bridge_f {
                routes = routes.or(m).unify().boxed();
            }

            if let Some(m) = frontend {
                routes = routes.or(m.into_warp_filter()).unify().boxed();
            }

            if let Some(m) = index_html_f {
                routes = routes.or(m).unify().boxed();
            }

            routes.with(log::custom(|info| {
                // We emit a custom span so it won't interfere with warp's default tracing event.
                tracing::info!(target: "stackable_backend::endpoint::trace",
                remote_addr = ?info.remote_addr(),
                method = %info.method(),
                path = info.path(),
                status = info.status().as_u16(),
                referer = ?info.referer(),
                user_agent = ?info.user_agent(),
                duration = info.elapsed().as_nanos());
            }))
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
