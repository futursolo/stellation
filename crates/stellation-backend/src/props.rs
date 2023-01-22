use std::marker::PhantomData;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use yew::Properties;

use crate::error::ServerAppResult;
use crate::Request;

/// The Properties provided to a server app.
#[derive(Properties, Debug)]
pub struct ServerAppProps<CTX, REQ> {
    request: Rc<REQ>,
    _marker: PhantomData<CTX>,
}

impl<CTX, REQ> ServerAppProps<CTX, REQ>
where
    REQ: Request<Context = CTX>,
{
    /// Returns the path of current request.
    pub fn path(&self) -> &str {
        self.request.path()
    }

    /// Returns queries of current request.
    pub fn queries<Q>(&self) -> ServerAppResult<Q>
    where
        Q: Serialize + for<'de> Deserialize<'de>,
    {
        self.request.queries()
    }

    /// Returns queries as a raw string.
    pub fn raw_queries(&self) -> &str {
        self.request.raw_queries()
    }

    /// Returns the current request context.
    pub fn context(&self) -> &CTX {
        self.request.context()
    }

    pub(crate) fn from_request(request: Rc<REQ>) -> Self {
        Self {
            request,
            _marker: PhantomData,
        }
    }
}

impl<REQ, CTX> PartialEq for ServerAppProps<REQ, CTX> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.request, &other.request)
    }
}

impl<REQ, CTX> Clone for ServerAppProps<REQ, CTX> {
    fn clone(&self) -> Self {
        Self {
            request: self.request.clone(),
            _marker: PhantomData,
        }
    }
}
