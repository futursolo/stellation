use http::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::ServerAppResult;

/// A trait that describes a request received by the backend.
pub trait Request {
    /// A request context that can be used to provide other information.
    type Context;

    /// Returns the path of current request.
    fn path(&self) -> &str;

    /// Returns queries as a raw string.
    fn raw_queries(&self) -> &str;

    /// Returns the headers of current request.
    fn headers(&self) -> &HeaderMap;

    /// Returns queries of current request.
    fn queries<Q>(&self) -> ServerAppResult<Q>
    where
        Q: Serialize + for<'de> Deserialize<'de>,
    {
        Ok(serde_urlencoded::from_str(self.raw_queries())?)
    }

    /// Returns the current request context.
    fn context(&self) -> &Self::Context;
}

/// A trait that describes a request for server-side rendering.
pub trait RenderRequest: Request {
    /// Returns the template of the html file.
    fn template(&self) -> &str;

    /// Returns true if this request should be rendered at the client side.
    fn is_client_only(&self) -> bool;
}
