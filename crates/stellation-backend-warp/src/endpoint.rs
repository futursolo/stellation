use core::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::Deref;

use bytes::Bytes;
use futures::future::LocalBoxFuture;
use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use http::status::StatusCode;
use stellation_backend::utils::ThreadLocalLazy;
use stellation_backend::{ServerAppProps, ServerRenderer};
use stellation_bridge::{Bridge, BridgeError, BridgeMetadata};
use tokio::sync::oneshot as sync_oneshot;
use warp::body::bytes;
use warp::path::FullPath;
use warp::reject::not_found;
use warp::reply::Response;
use warp::ws::{Message, Ws};
use warp::{header, log, reply, Filter, Rejection, Reply};
use yew::platform::{LocalHandle, Runtime};
use yew::prelude::*;

use crate::frontend::Frontend;
use crate::request::WarpRequest;
use crate::{html, SERVER_ID};

type BoxedSendFn<IN, OUT> = Box<dyn Send + Fn(IN) -> LocalBoxFuture<'static, OUT>>;
type SendFn<IN, OUT> = ThreadLocalLazy<BoxedSendFn<IN, OUT>>;

/// Creates a stellation endpoint that can be turned into a wrap filter.
///
/// This endpoint serves bridge requests and frontend requests.
/// You can turn this type into a tower service by calling [`into_warp_filter()`].
pub struct WarpEndpoint<COMP, CTX = (), BCTX = ()>
where
    COMP: BaseComponent,
{
    frontend: Option<crate::frontend::Frontend>,
    affix_context: SendFn<WarpRequest<()>, WarpRequest<CTX>>,

    bridge: Option<Bridge>,
    affix_bridge_context: SendFn<BridgeMetadata<()>, BridgeMetadata<BCTX>>,

    auto_refresh: bool,
    _marker: PhantomData<COMP>,
}

impl<COMP, CTX, BCTX> fmt::Debug for WarpEndpoint<COMP, CTX, BCTX>
where
    COMP: BaseComponent,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WarpEndpoint<_>")
    }
}

impl<COMP, CTX, BCTX> Default for WarpEndpoint<COMP, CTX, BCTX>
where
    COMP: BaseComponent,
    CTX: 'static + Default,
    BCTX: 'static + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<COMP, CTX, BCTX> WarpEndpoint<COMP, CTX, BCTX>
where
    COMP: BaseComponent,
    CTX: 'static,
    BCTX: 'static,
{
    /// Creates an endpoint.
    pub fn new() -> Self
    where
        CTX: Default,
        BCTX: Default,
    {
        Self {
            affix_context: SendFn::<WarpRequest<()>, WarpRequest<CTX>>::new(move || {
                Box::new(|m| async move { m.with_context(CTX::default()) }.boxed())
            }),
            affix_bridge_context: SendFn::<BridgeMetadata<()>, BridgeMetadata<BCTX>>::new(
                move || Box::new(|m| async move { m.with_context(BCTX::default()) }.boxed()),
            ),
            bridge: None,
            frontend: None,
            auto_refresh: false,
            _marker: PhantomData,
        }
    }

    /// Appends a context to current request.
    pub fn with_append_context<F, C, Fut>(self, append_context: F) -> WarpEndpoint<COMP, C, BCTX>
    where
        F: 'static + Clone + Send + Fn(WarpRequest<()>) -> Fut,
        Fut: 'static + Future<Output = WarpRequest<C>>,
        C: 'static,
    {
        WarpEndpoint {
            affix_context: SendFn::<WarpRequest<()>, WarpRequest<C>>::new(move || {
                let append_context = append_context.clone();
                Box::new(move |input| append_context(input).boxed_local())
            }),
            affix_bridge_context: self.affix_bridge_context,
            bridge: self.bridge,
            frontend: self.frontend,
            auto_refresh: self.auto_refresh,
            _marker: PhantomData,
        }
    }

    /// Appends a bridge context to current request.
    pub fn with_append_bridge_context<F, C, Fut>(
        self,
        append_bridge_context: F,
    ) -> WarpEndpoint<COMP, CTX, C>
    where
        F: 'static + Clone + Send + Fn(BridgeMetadata<()>) -> Fut,
        Fut: 'static + Future<Output = BridgeMetadata<C>>,
        C: 'static,
    {
        WarpEndpoint {
            affix_context: self.affix_context,
            affix_bridge_context: SendFn::<BridgeMetadata<()>, BridgeMetadata<C>>::new(move || {
                let append_bridge_context = append_bridge_context.clone();
                Box::new(move |input| append_bridge_context(input).boxed_local())
            }),
            bridge: self.bridge,
            frontend: self.frontend,
            auto_refresh: self.auto_refresh,
            _marker: PhantomData,
        }
    }

    /// Serves a bridge on current endpoint.
    pub fn with_bridge(mut self, bridge: Bridge) -> Self {
        self.bridge = Some(bridge);
        self
    }

    /// Enables auto refresh.
    ///
    /// This is useful during development.
    pub fn with_auto_refresh(mut self) -> Self {
        self.auto_refresh = true;

        self
    }

    /// Serves a frontend with current endpoint.
    pub fn with_frontend(mut self, frontend: Frontend) -> Self {
        self.frontend = Some(frontend);

        self
    }
}

impl<COMP, CTX, BCTX> WarpEndpoint<COMP, CTX, BCTX>
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX, WarpRequest<CTX>>>,
    CTX: 'static,
    BCTX: 'static,
{
    fn create_index_filter(
        &self,
    ) -> Option<
        impl Clone
            + Send
            + Filter<
                Extract = (Response,),
                Error = Rejection,
                Future = impl Future<Output = Result<(Response,), Rejection>>,
            >,
    > {
        let index_html = self.frontend.as_ref()?.index_html();
        let affix_context = self.affix_context.clone();
        let bridge = self.bridge.clone().unwrap_or_default();
        let auto_refresh = self.auto_refresh;
        let affix_bridge_context = self.affix_bridge_context.clone();

        let create_render_inner = move |req, tx: sync_oneshot::Sender<String>| async move {
            let req = (affix_context.deref())(req).await;
            let bridge_metadata = (affix_bridge_context.deref())(BridgeMetadata::new()).await;

            let s = ServerRenderer::<COMP, WarpRequest<CTX>, CTX>::new(req)
                .bridge(bridge, bridge_metadata)
                .render()
                .await;

            let _ = tx.send(s);
        };

        let render_html = move |req| async move {
            let (tx, rx) = sync_oneshot::channel::<String>();

            // We spawn into a local runtime early for higher efficiency.
            match LocalHandle::try_current() {
                Some(handle) => handle.spawn_local(create_render_inner(req, tx)),
                // TODO: Allow Overriding Runtime with Endpoint.
                None => Runtime::default().spawn_pinned(move || create_render_inner(req, tx)),
            }

            warp::reply::html(rx.await.expect("renderer panicked?"))
        };

        let f = warp::get()
            .and(warp::path::full())
            .and(
                warp::query::raw().or_else(|_| async move { Ok::<_, Rejection>((String::new(),)) }),
            )
            .then(move |path: FullPath, raw_queries| {
                let render_html = render_html.clone();
                let index_html = index_html.clone();

                async move {
                    let mut template = index_html.read_content().await;
                    if auto_refresh {
                        template = html::add_refresh_script(&template).into();
                    }

                    let req = WarpRequest {
                        path,
                        raw_queries,
                        template,
                        context: (),
                    };

                    render_html(req).await.into_response()
                }
            });

        Some(f)
    }

    fn create_refresh_filter(
    ) -> impl Clone + Send + Filter<Extract = (Response,), Error = Rejection> {
        warp::path::path("_refresh")
            .and(warp::ws())
            .then(|m: Ws| async move {
                m.on_upgrade(|mut ws| async move {
                    let read_refresh = {
                        || async move {
                            while let Some(m) = ws.next().await {
                                let m = match m {
                                    Ok(m) => m,
                                    Err(e) => {
                                        tracing::error!("receive message error: {:?}", e);

                                        if let Err(e) = ws.close().await {
                                            tracing::error!("failed to close websocket: {:?}", e);
                                        }

                                        return;
                                    }
                                };

                                if m.is_ping() || m.is_pong() {
                                    continue;
                                }

                                let m = match m.to_str() {
                                    Ok(m) => m,
                                    Err(_) => {
                                        tracing::error!("received unknown message: {:?}", m);
                                        return;
                                    }
                                };

                                // Ping client if string matches.
                                // Otherwise, tell the client to reload the page.
                                let message_to_send = if m == SERVER_ID.as_str() {
                                    Message::ping("")
                                } else {
                                    Message::text("restart")
                                };

                                if let Err(e) = ws.send(message_to_send).await {
                                    tracing::error!("error sending message: {:?}", e);
                                    return;
                                }
                            }
                        }
                    };

                    match LocalHandle::try_current() {
                        Some(handle) => handle.spawn_local(read_refresh()),
                        // TODO: Allow Overriding Runtime with Endpoint.
                        None => Runtime::default().spawn_pinned(read_refresh),
                    }
                })
                .into_response()
            })
    }

    fn create_bridge_filter(
        &self,
    ) -> Option<impl Clone + Send + Filter<Extract = (Response,), Error = Rejection>> {
        let bridge = self.bridge.clone()?;

        let http_bridge_f = warp::post()
            .and(header::exact_ignore_case(
                "content-type",
                "application/x-bincode",
            ))
            .and(header::optional("authorization"))
            .and(bytes())
            .then(move |token: Option<String>, input: Bytes| {
                let bridge = bridge.clone();
                let (tx, rx) = sync_oneshot::channel();

                let resolve_encoded = move || async move {
                    let mut meta = BridgeMetadata::<()>::new();

                    if let Some(m) = token {
                        if !m.starts_with("Bearer ") {
                            let reply =
                                reply::with_status("", StatusCode::BAD_REQUEST).into_response();

                            let _ = tx.send(reply);
                            return;
                        }

                        meta = meta.with_token(m.split_at(7).1);
                    }

                    let content = bridge
                        .connect(meta)
                        .and_then(|m| async move { m.resolve_encoded(&input).await })
                        .await;

                    let reply = match content {
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
                    };

                    let _ = tx.send(reply);
                };

                match LocalHandle::try_current() {
                    Some(handle) => handle.spawn_local(resolve_encoded()),
                    // TODO: Allow Overriding Runtime with Endpoint.
                    None => Runtime::default().spawn_pinned(resolve_encoded),
                }

                async move { rx.await.expect("failed to resolve the bridge request") }
            });

        Some(warp::path::path("_bridge").and(http_bridge_f))
    }

    /// Creates a warp filter from current endpoint.
    pub fn into_warp_filter(
        self,
    ) -> impl Clone + Send + Filter<Extract = (impl Reply + Send,), Error = Rejection> {
        let bridge_f = self.create_bridge_filter();
        let index_html_f = self.create_index_filter();

        let Self { frontend, .. } = self;

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

        if self.auto_refresh {
            routes = routes.or(Self::create_refresh_filter()).unify().boxed();
        }

        if let Some(m) = index_html_f {
            routes = routes.or(m).unify().boxed();
        }

        routes.with(log::custom(|info| {
            // We emit a custom span so it won't interfere with warp's default tracing event.
            tracing::info!(target: "stellation_backend::endpoint::trace",
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
