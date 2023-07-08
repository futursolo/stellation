//! States used by the frontend and backend.
//!
//! These states are registered automatically if you use backend endpoint or frontend renderer.

use bounce::Atom;

use crate::links::Link;
use crate::Bridge;

/// The bridge state.
#[derive(Atom, Debug)]
pub struct BridgeState<L>
where
    L: Link,
{
    /// The bridge stored in the state.
    pub inner: Option<Bridge<L>>,
}

impl<L> Default for BridgeState<L>
where
    L: Link,
{
    fn default() -> Self {
        Self { inner: None }
    }
}

impl<L> PartialEq for BridgeState<L>
where
    L: Link,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<L> Eq for BridgeState<L> where L: Link {}
