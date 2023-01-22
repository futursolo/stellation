use serde::{Deserialize, Serialize};

use crate::ServerAppResult;

/// A trait that describes a request for server-side rendering.
pub trait Request {
    /// A request context that can be used to provide other information.
    type Context;

    /// Returns the template of the html file.
    fn template(&self) -> &str;

    /// Returns the path of current request.
    fn path(&self) -> &str;

    /// Returns queries as a raw string.
    fn raw_queries(&self) -> &str;

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
