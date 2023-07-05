use core::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::Deref;

use bytes::Bytes;
use futures::future::LocalBoxFuture;
use futures::{FutureExt, SinkExt, StreamExt};
use http::status::StatusCode;
use stellation_backend::utils::ThreadLocalLazy;
use stellation_backend::{ServerAppProps, ServerRenderer};
use stellation_bridge::links::{Link, PhantomLink};
use stellation_bridge::{Bridge, BridgeError};
use tokio::sync::oneshot as sync_oneshot;
use warp::body::bytes;
use warp::reply::Response;
use warp::ws::{Message, Ws};
use warp::{header, log, reply, Filter, Rejection, Reply};
use yew::platform::{LocalHandle, Runtime};
use yew::prelude::*;

use crate::filters::{reject, warp_request};
use crate::frontend::{Frontend, IndexHtml};
use crate::request::WarpRequest;
use crate::utils::spawn_pinned_or_local;
use crate::SERVER_ID;

type BoxedSendFn<IN, OUT> = Box<dyn Send + Fn(IN) -> LocalBoxFuture<'static, OUT>>;
type SendFn<IN, OUT> = ThreadLocalLazy<BoxedSendFn<IN, OUT>>;

type AppendContext<CTX> = SendFn<WarpRequest<()>, WarpRequest<CTX>>;
type CreateBridge<L> = SendFn<WarpRequest<()>, Bridge<L>>;

type RenderIndex = SendFn<WarpRequest<()>, String>;

/// Creates a stellation endpoint that can be turned into a warp filter.
///
/// This endpoint serves bridge requests and frontend requests.
/// You can turn this type into a tower service by calling [`into_warp_filter()`].
pub struct WarpEndpoint<COMP, CTX = (), L = PhantomLink>
where
    COMP: BaseComponent,
{
    frontend: Option<crate::frontend::Frontend>,
    append_context: AppendContext<CTX>,
    create_bridge: Option<CreateBridge<L>>,
    auto_refresh: bool,
    _marker: PhantomData<COMP>,
}

impl<COMP, CTX, L> fmt::Debug for WarpEndpoint<COMP, CTX, L>
where
    COMP: BaseComponent,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WarpEndpoint<_>")
    }
}

impl<COMP, CTX> Default for WarpEndpoint<COMP, CTX>
where
    COMP: BaseComponent,
    CTX: 'static + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<COMP, CTX> WarpEndpoint<COMP, CTX>
where
    COMP: BaseComponent,
    CTX: 'static,
{
    /// Creates an endpoint.
    pub fn new() -> Self
    where
        CTX: Default,
    {
        Self {
            append_context: AppendContext::<CTX>::new(move || {
                Box::new(|m| async move { m.with_context(CTX::default()) }.boxed())
            }),
            frontend: None,
            create_bridge: None,
            auto_refresh: false,
            _marker: PhantomData,
        }
    }
}

impl<COMP, CTX, L> WarpEndpoint<COMP, CTX, L>
where
    COMP: BaseComponent,
    CTX: 'static,
{
    /// Appends a context to current request.
    pub fn with_append_context<F, C, Fut>(self, append_context: F) -> WarpEndpoint<COMP, C, L>
    where
        F: 'static + Clone + Send + Fn(WarpRequest<()>) -> Fut,
        Fut: 'static + Future<Output = WarpRequest<C>>,
        C: 'static,
    {
        WarpEndpoint {
            append_context: AppendContext::<C>::new(move || {
                let append_context = append_context.clone();
                Box::new(move |input| append_context(input).boxed_local())
            }),
            frontend: self.frontend,
            create_bridge: self.create_bridge,
            auto_refresh: self.auto_refresh,
            _marker: PhantomData,
        }
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

    /// Appends a bridge to current request.
    pub fn with_create_bridge<F, LINK, Fut>(self, create_bridge: F) -> WarpEndpoint<COMP, CTX, LINK>
    where
        F: 'static + Clone + Send + Fn(WarpRequest<()>) -> Fut,
        Fut: 'static + Future<Output = Bridge<LINK>>,
        LINK: 'static + Link,
    {
        WarpEndpoint {
            append_context: self.append_context,
            frontend: self.frontend,
            create_bridge: Some(CreateBridge::new(move || {
                let create_bridge = create_bridge.clone();
                Box::new(move |input| create_bridge(input).boxed_local())
            })),
            auto_refresh: self.auto_refresh,
            _marker: PhantomData,
        }
    }
}

impl<COMP, CTX, L> WarpEndpoint<COMP, CTX, L>
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX, WarpRequest<CTX>>>,
    CTX: 'static,
    L: 'static + Link,
{
    fn create_render_index(&self) -> RenderIndex {
        let append_context = self.append_context.clone();

        match self.create_bridge.clone() {
            Some(create_bridge) => RenderIndex::new(move || {
                let append_context = append_context.clone();
                let create_bridge = create_bridge.clone();

                Box::new(move |req| {
                    let append_context = append_context.clone();
                    let create_bridge = create_bridge.clone();
                    async move {
                        let bridge = create_bridge(req.clone()).await;
                        let req = (append_context.deref())(req).await;

                        ServerRenderer::<COMP, WarpRequest<CTX>, CTX>::new(req)
                            .bridge(bridge)
                            .render()
                            .await
                    }
                    .boxed_local()
                })
            }),
            None => RenderIndex::new(move || {
                let append_context = append_context.clone();
                Box::new(move |req| {
                    let append_context = append_context.clone();
                    async move {
                        let req = (append_context.deref())(req).await;

                        ServerRenderer::<COMP, WarpRequest<CTX>, CTX>::new(req)
                            .render()
                            .await
                    }
                    .boxed_local()
                })
            }),
        }
    }

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
        let render_index = self.create_render_index();
        let index_html = self.frontend.as_ref()?.index_html();
        let auto_refresh = self.auto_refresh;

        let f = warp::get()
            .and(warp_request(index_html, auto_refresh))
            .then(move |req: WarpRequest<()>| {
                let render_index = render_index.clone();

                async move {
                    let (tx, rx) = sync_oneshot::channel::<String>();
                    spawn_pinned_or_local(move || async move {
                        let s = render_index(req).await;

                        let _ = tx.send(s);
                    });

                    warp::reply::html(rx.await.expect("renderer panicked?")).into_response()
                }
            });

        Some(f)
    }

    fn create_refresh_filter(
        &self,
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
        let create_bridge = self.create_bridge.clone()?;

        let index_html = self
            .frontend
            .as_ref()
            .map(|m| m.index_html())
            .unwrap_or_else(IndexHtml::fallback);
        let auto_refresh = self.auto_refresh;

        let http_bridge_f = warp::post()
            .and(header::exact_ignore_case(
                "content-type",
                "application/x-bincode",
            ))
            .and(warp_request(index_html, auto_refresh))
            .and(bytes())
            .then(move |req: WarpRequest<()>, input: Bytes| {
                let create_bridge = create_bridge.clone();

                let (tx, rx) = sync_oneshot::channel();

                let resolve_encoded = move || async move {
                    let bridge = create_bridge(req).await;

                    let content = bridge.link().resolve_encoded(&input).await;

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

                spawn_pinned_or_local(resolve_encoded);

                async move { rx.await.expect("failed to resolve the bridge request") }
            });

        Some(warp::path::path("_bridge").and(http_bridge_f))
    }

    /// Creates a warp filter from current endpoint.
    pub fn into_warp_filter(
        self,
    ) -> impl Clone + Send + Filter<Extract = (impl Reply + Send,), Error = Rejection> {
        let index_html_f = self.create_index_filter();
        let auto_refresh_f = self.auto_refresh.then(|| self.create_refresh_filter());
        let bridge_f = self.create_bridge_filter();

        let Self { frontend, .. } = self;

        let frontend_f = frontend.map(|m| m.into_warp_filter());

        let routes =
        // Bridge goes first
        bridge_f
            .map(|m| m.boxed())
            .into_iter()
            // Then auto refresh
            .chain(auto_refresh_f.map(|m| m.boxed()).into_iter())
            // Make sure "/" is rendered as index.html
            .chain(
                index_html_f
                    .clone()
                    .map(|m| warp::path::end().and(m).boxed())
                    .into_iter(),
            )
            // Serve other resources, if available.
            .chain(frontend_f.map(|m| m.boxed()).into_iter())
            // Fallback to index.html
            .chain(index_html_f.map(|m| m.boxed()).into_iter())
            .fold(reject().boxed(), |last, item| last.or(item).unify().boxed());

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
