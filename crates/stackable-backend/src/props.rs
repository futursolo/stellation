use std::sync::Arc;

use serde::{Deserialize, Serialize};
use yew::Properties;

use crate::error::ServerAppResult;

#[derive(Debug)]
#[non_exhaustive]
enum Path {
    #[cfg(feature = "warp-filter")]
    Warp(warp::path::FullPath),
}

impl Path {
    fn as_str(&self) -> &str {
        match self {
            #[cfg(feature = "warp-filter")]
            Self::Warp(m) => m.as_str(),
            #[cfg(not(feature = "warp-filter"))]
            _ => panic!("not implemented variant"),
        }
    }
}

#[derive(Debug)]
pub struct Inner {
    path: Path,
    raw_queries: String,
}

/// The Properties provided to a server app.
#[derive(Properties, Debug)]
pub struct ServerAppProps<T = ()> {
    inner: Arc<Inner>,
    context: Arc<T>,
    client_only: bool,
}

impl<T> ServerAppProps<T> {
    /// Returns the path of current request.
    pub fn path(&self) -> &str {
        self.inner.path.as_str()
    }

    /// Returns queries of current request.
    pub fn queries<Q>(&self) -> ServerAppResult<Q>
    where
        Q: Serialize + for<'de> Deserialize<'de>,
    {
        Ok(serde_urlencoded::from_str(&self.inner.raw_queries)?)
    }

    /// Returns queries as a raw string.
    pub fn raw_queries(&self) -> &str {
        &self.inner.raw_queries
    }

    /// Returns the current request context.
    pub fn context(&self) -> &T {
        &self.context
    }
}

impl<T> PartialEq for ServerAppProps<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner) && Arc::ptr_eq(&self.context, &other.context)
    }
}

impl<T> Clone for ServerAppProps<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            context: self.context.clone(),
            client_only: self.client_only,
        }
    }
}

impl<T> ServerAppProps<T> {
    // Appends a context to current server app to help resolving the request.
    pub fn with_context<CTX>(self, context: CTX) -> ServerAppProps<CTX> {
        ServerAppProps {
            inner: self.inner,
            context: context.into(),
            client_only: false,
        }
    }

    /// Excludes this request from server-side rendering.
    pub fn client_only(mut self) -> Self {
        self.client_only = true;
        self
    }

    #[cfg(feature = "warp-filter")]
    pub(crate) fn is_client_only(&self) -> bool {
        self.client_only
    }
}

#[cfg(feature = "warp-filter")]
mod feat_warp_filter {
    use warp::path::FullPath;

    use super::*;

    impl ServerAppProps<()> {
        pub(crate) fn from_warp_request(path: FullPath, raw_queries: String) -> Self {
            Self {
                inner: Inner {
                    path: Path::Warp(path),
                    raw_queries,
                }
                .into(),
                context: ().into(),
                client_only: false,
            }
        }
    }
}
