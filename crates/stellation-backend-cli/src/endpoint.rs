use std::convert::Infallible;

use hyper::{Body, Request, Response};
use stellation_backend::ServerAppProps;
use stellation_backend_tower::{Frontend, TowerEndpoint, TowerEndpointWithBridge, TowerRequest};
use stellation_bridge::links::Link;
use tower::util::BoxCloneService;
use tower::ServiceExt;
use yew::BaseComponent;

mod sealed {
    use stellation_backend_tower::{TowerEndpoint, TowerEndpointWithBridge};
    use yew::BaseComponent;

    pub trait Sealed {}

    impl<COMP, CTX> Sealed for TowerEndpoint<COMP, CTX> where COMP: BaseComponent {}
    impl<COMP, CTX, L> Sealed for TowerEndpointWithBridge<COMP, CTX, L> where COMP: BaseComponent {}
}

pub trait SealedEndpointBase: sealed::Sealed {
    fn with_frontend(self, frontend: Frontend) -> Self;
    fn with_auto_refresh(self) -> Self;
    fn into_tower_service(self) -> BoxCloneService<Request<Body>, Response<Body>, Infallible>;
}

impl<COMP, CTX> SealedEndpointBase for TowerEndpoint<COMP, CTX>
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX, TowerRequest<CTX>>>,
    CTX: 'static,
{
    fn with_frontend(self, frontend: Frontend) -> Self {
        TowerEndpoint::with_frontend(self, frontend)
    }

    fn with_auto_refresh(self) -> Self {
        TowerEndpoint::with_auto_refresh(self)
    }

    fn into_tower_service(self) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
        TowerEndpoint::into_tower_service(self).boxed_clone()
    }
}

impl<COMP, CTX, L> SealedEndpointBase for TowerEndpointWithBridge<COMP, CTX, L>
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX, TowerRequest<CTX>>>,
    CTX: 'static,
    L: 'static + Link,
{
    fn with_frontend(self, frontend: Frontend) -> Self {
        TowerEndpointWithBridge::with_frontend(self, frontend)
    }

    fn with_auto_refresh(self) -> Self {
        TowerEndpointWithBridge::with_auto_refresh(self)
    }

    fn into_tower_service(self) -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
        TowerEndpointWithBridge::into_tower_service(self).boxed_clone()
    }
}
