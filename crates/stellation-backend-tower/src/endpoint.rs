use std::convert::Infallible;
use std::future::Future;

use hyper::{Body, Request, Response};
use stellation_backend::ServerAppProps;
use stellation_backend_warp::{Frontend, WarpEndpoint};
use stellation_bridge::links::{Link, PhantomLink};
use stellation_bridge::Bridge;
use tower::Service;
use yew::BaseComponent;

use crate::{TowerRenderRequest, TowerRequest};

/// Creates a stellation endpoint that can be turned into a tower service.
///
/// This endpoint serves bridge requests and frontend requests.
/// You can turn this type into a tower service by calling
/// [`into_tower_service()`](Self::into_tower_service).
#[derive(Debug)]
pub struct TowerEndpoint<COMP, CTX = (), L = PhantomLink>
where
    COMP: BaseComponent,
{
    inner: WarpEndpoint<COMP, CTX, L>,
}

impl<COMP, CTX> Default for TowerEndpoint<COMP, CTX>
where
    COMP: BaseComponent,
    CTX: 'static + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<COMP, CTX> TowerEndpoint<COMP, CTX>
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
            inner: WarpEndpoint::default(),
        }
    }
}

impl<COMP, CTX, L> TowerEndpoint<COMP, CTX, L>
where
    COMP: BaseComponent,
    CTX: 'static,
{
    /// Appends a context to current request.
    pub fn with_append_context<F, C, Fut>(self, append_context: F) -> TowerEndpoint<COMP, C, L>
    where
        F: 'static + Clone + Send + Fn(TowerRenderRequest<()>) -> Fut,
        Fut: 'static + Future<Output = TowerRenderRequest<C>>,
        C: 'static,
    {
        TowerEndpoint {
            inner: self.inner.with_append_context(append_context),
        }
    }

    /// Appends a bridge to current request.
    pub fn with_create_bridge<F, LINK, Fut>(
        self,
        create_bridge: F,
    ) -> TowerEndpoint<COMP, CTX, LINK>
    where
        F: 'static + Clone + Send + Fn(TowerRequest<()>) -> Fut,
        Fut: 'static + Future<Output = Bridge<LINK>>,
        LINK: 'static + Link,
    {
        TowerEndpoint {
            inner: self.inner.with_create_bridge(create_bridge),
        }
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
    >
    where
        COMP: BaseComponent<Properties = ServerAppProps<CTX, TowerRenderRequest<CTX>>>,
        CTX: 'static,
        L: 'static + Link,
    {
        let routes = self.inner.into_warp_filter();
        warp::service(routes)
    }
}
