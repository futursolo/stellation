use std::sync::Arc;

use http::HeaderMap;
use stellation_backend::Request;
use warp::path::FullPath;

/// A stellation request with information extracted with warp filters.
#[derive(Debug)]
pub struct WarpRequest<CTX> {
    pub(crate) path: FullPath,
    pub(crate) raw_queries: String,
    pub(crate) template: Arc<str>,
    pub(crate) context: CTX,
    pub(crate) headers: HeaderMap,
}

impl<CTX> Request for WarpRequest<CTX> {
    type Context = CTX;

    fn path(&self) -> &str {
        self.path.as_str()
    }

    fn raw_queries(&self) -> &str {
        &self.raw_queries
    }

    fn template(&self) -> &str {
        self.template.as_ref()
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
            template: self.template,
            headers: self.headers,
            context,
        }
    }
}
