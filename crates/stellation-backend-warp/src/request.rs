use std::sync::Arc;

use http::HeaderMap;
use stellation_backend::{RenderRequest, Request};
use warp::path::FullPath;

/// A stellation request with information extracted from a warp request, used by
/// server-side-rendering.
#[derive(Debug)]
pub struct WarpRenderRequest<CTX> {
    pub(crate) inner: WarpRequest<CTX>,
    pub(crate) template: Arc<str>,
}

impl<CTX> Clone for WarpRenderRequest<CTX> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            template: self.template.clone(),
        }
    }
}

impl<CTX> Request for WarpRenderRequest<CTX> {
    type Context = CTX;

    fn path(&self) -> &str {
        self.inner.path()
    }

    fn raw_queries(&self) -> &str {
        self.inner.raw_queries()
    }

    fn context(&self) -> &Self::Context {
        self.inner.context()
    }

    fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }
}

impl<CTX> RenderRequest for WarpRenderRequest<CTX> {
    fn template(&self) -> &str {
        self.template.as_ref()
    }
}

impl<CTX> WarpRenderRequest<CTX> {
    /// Appends a context to current server app to help resolving the request.
    pub fn with_context<C>(self, context: C) -> WarpRenderRequest<C> {
        WarpRenderRequest {
            template: self.template,
            inner: self.inner.with_context(context),
        }
    }

    pub(crate) fn into_inner(self) -> WarpRequest<CTX> {
        self.inner
    }
}

/// A stellation request with information extracted from a warp request, used by
/// server-side-rendering.
#[derive(Debug)]
pub struct WarpRequest<CTX> {
    pub(crate) path: Arc<FullPath>,
    pub(crate) raw_queries: Arc<str>,
    pub(crate) context: Arc<CTX>,
    pub(crate) headers: HeaderMap,
}

impl<CTX> Clone for WarpRequest<CTX> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            raw_queries: self.raw_queries.clone(),
            context: self.context.clone(),
            headers: self.headers.clone(),
        }
    }
}

impl<CTX> Request for WarpRequest<CTX> {
    type Context = CTX;

    fn path(&self) -> &str {
        self.path.as_str()
    }

    fn raw_queries(&self) -> &str {
        &self.raw_queries
    }

    fn context(&self) -> &Self::Context {
        &self.context
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}

impl<CTX> WarpRequest<CTX> {
    /// Appends a context to current server app to help resolving the request.
    pub fn with_context<C>(self, context: C) -> WarpRequest<C> {
        WarpRequest {
            path: self.path,
            raw_queries: self.raw_queries,
            headers: self.headers,
            context: context.into(),
        }
    }
}
