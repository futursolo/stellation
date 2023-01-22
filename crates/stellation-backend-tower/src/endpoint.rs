use std::convert::Infallible;
use std::future::Future;

use hyper::{Body, Request, Response};
use stellation_backend::ServerAppProps;
use stellation_backend_warp::{Frontend, WarpEndpoint};
use stellation_bridge::{Bridge, BridgeMetadata};
use tower::Service;
use yew::BaseComponent;

use crate::TowerRequest;

/// Creates a stellation endpoint that can be turned into a tower service.
///
/// This endpoint serves bridge requests and frontend requests.
/// You can turn this type into a tower service by calling [`into_tower_service()`].
#[derive(Debug)]
pub struct TowerEndpoint<COMP, CTX = (), BCTX = ()>
where
    COMP: BaseComponent,
{
    inner: WarpEndpoint<COMP, CTX, BCTX>,
}

impl<COMP, CTX, BCTX> Default for TowerEndpoint<COMP, CTX, BCTX>
where
    COMP: BaseComponent,
    CTX: 'static + Default,
    BCTX: 'static + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<COMP, CTX, BCTX> TowerEndpoint<COMP, CTX, BCTX>
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
            inner: WarpEndpoint::default(),
        }
    }

    /// Appends a context to current request.
    pub fn with_append_context<F, C, Fut>(self, append_context: F) -> TowerEndpoint<COMP, C, BCTX>
    where
        F: 'static + Clone + Send + Fn(TowerRequest<()>) -> Fut,
        Fut: 'static + Future<Output = TowerRequest<C>>,
        C: 'static,
    {
        TowerEndpoint {
            inner: self.inner.with_append_context(append_context),
        }
    }

    /// Appends a bridge context to current request.
    pub fn with_append_bridge_context<F, C, Fut>(
        self,
        append_bridge_context: F,
    ) -> TowerEndpoint<COMP, CTX, C>
    where
        F: 'static + Clone + Send + Fn(BridgeMetadata<()>) -> Fut,
        Fut: 'static + Future<Output = BridgeMetadata<C>>,
        C: 'static,
    {
        TowerEndpoint {
            inner: self.inner.with_append_bridge_context(append_bridge_context),
        }
    }

    /// Serves a bridge on current endpoint.
    pub fn with_bridge(mut self, bridge: Bridge) -> Self {
        self.inner = self.inner.with_bridge(bridge);
        self
    }

    /// Enables auto refresh.
    ///
    /// This is useful during development.
    pub fn with_auto_refresh(mut self) -> Self {
        self.inner = self.inner.with_auto_refresh();
        self
    }

    /// Serves a frontend with current endpoint.
    pub fn with_frontend(mut self, frontend: Frontend) -> Self {
        self.inner = self.inner.with_frontend(frontend);
        self
    }
}

impl<COMP, CTX, BCTX> TowerEndpoint<COMP, CTX, BCTX>
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX, TowerRequest<CTX>>>,
    CTX: 'static,
    BCTX: 'static,
{
    /// Creates a tower service from current endpoint.
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
        let routes = self.inner.into_warp_filter();
        warp::service(routes)
    }
}
