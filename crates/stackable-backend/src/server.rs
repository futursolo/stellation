use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;

use futures::TryStream;
use hyper::body::HttpBody;
use hyper::server::accept::Accept;
use hyper::server::conn::AddrIncoming;
use hyper::{Body, Request, Response};
use tokio::io::{AsyncRead, AsyncWrite};
use tower_service::Service;
use yew::platform::Runtime;

// An executor to process requests on the Yew runtime.
//
// By spawning requests on the Yew runtime,
// it processes request on the same thread as the rendering task.
//
// This increases performance in some environments (e.g.: in VM).
#[derive(Clone, Default)]
struct Executor {
    inner: Runtime,
}

impl<F> hyper::rt::Executor<F> for Executor
where
    F: Future + Send + 'static,
{
    fn execute(&self, fut: F) {
        self.inner.spawn_pinned(move || async move {
            fut.await;
        });
    }
}

#[derive(Debug)]
pub struct Server<I> {
    inner: hyper::server::Builder<I>,
    rt: Option<Runtime>,
}

impl<I> Server<I> {
    pub fn bind(addr: impl Into<SocketAddr> + 'static) -> Server<AddrIncoming> {
        Server {
            inner: hyper::server::Server::bind(&addr.into()),
            rt: None,
        }
    }

    pub fn from_stream<S, A, T, E>(stream: S) -> Server<impl Accept<Conn = T, Error = E>>
    where
        S: TryStream<Ok = T, Error = E, Item = Result<T, E>> + Send,
        T: AsyncRead + AsyncWrite + Send + 'static + Unpin,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Server {
            inner: hyper::server::Server::builder(hyper::server::accept::from_stream(stream)),
            rt: None,
        }
    }
}
impl<I> Server<I>
where
    I: Accept,
    I::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    I::Conn: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    pub async fn serve_service<HS, HF, HE, B, BD, BE>(self, svc: HS) -> hyper::Result<()>
    where
        HS: Service<Request<Body>, Response = Response<B>, Future = HF, Error = HE>
            + Send
            + Clone
            + 'static,
        HE: Into<Box<dyn std::error::Error + Send + Sync>>,
        HF: Future<Output = Result<Response<B>, HE>> + Send + 'static,

        B: HttpBody<Data = BD, Error = BE> + Send + 'static,
        BD: Send + 'static,
        BE: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let make_svc = hyper::service::make_service_fn(move |_| {
            let svc = svc.clone();
            async move { Ok::<_, Infallible>(svc.clone()) }
        });

        self.serve_make_service(make_svc).await
    }

    pub async fn serve_make_service<MS, ME, MF, HS, HF, HE, B, BD, BE>(
        self,
        make_svc: MS,
    ) -> hyper::Result<()>
    where
        MS: for<'a> Service<&'a I::Conn, Response = HS, Error = ME, Future = MF>,
        ME: Into<Box<dyn std::error::Error + Send + Sync>>,
        MF: Future<Output = Result<HS, ME>> + Send + 'static,

        HS: Service<Request<Body>, Response = Response<B>, Future = HF, Error = HE>
            + Send
            + 'static,
        HE: Into<Box<dyn std::error::Error + Send + Sync>>,
        HF: Future<Output = Result<Response<B>, HE>> + Send + 'static,

        B: HttpBody<Data = BD, Error = BE> + Send + 'static,
        BD: Send + 'static,
        BE: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let Self { inner, rt } = self;

        inner
            .executor(Executor {
                inner: rt.unwrap_or_default(),
            })
            .serve(make_svc)
            .await
    }
}
