//! States used by the frontend and backend.
//!
//! These states are registered automatically if you use backend endpoint or frontend renderer.

use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use bounce::{Atom, BounceStates, Selector};

use crate::links::Link;
use crate::Bridge;

type SelectBridge<L> = Rc<dyn Fn(&BounceStates) -> Bridge<L>>;

enum BridgeStateVariant<L>
where
    L: Link,
{
    Value(Bridge<L>),
    Selector(SelectBridge<L>),
}

impl<L> fmt::Debug for BridgeStateVariant<L>
where
    L: Link,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BridgeStateVariant<_>")
            .finish_non_exhaustive()
    }
}

impl<L> PartialEq for BridgeStateVariant<L>
where
    L: Link,
{
    // We allow this implementation for now as we do not expect this state to change after it is
    // declared.
    #[allow(clippy::vtable_address_comparisons)]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Value(ref l), Self::Value(ref r)) => l == r,
            (Self::Selector(ref l), Self::Selector(ref r)) => Rc::ptr_eq(l, r),
            _ => false,
        }
    }
}

impl<L> Eq for BridgeStateVariant<L> where L: Link {}
impl<L> Clone for BridgeStateVariant<L>
where
    L: Link,
{
    fn clone(&self) -> Self {
        match self {
            Self::Value(v) => Self::Value(v.clone()),
            Self::Selector(s) => Self::Selector(s.clone()),
        }
    }
}

/// The bridge state.
#[derive(Atom, Debug)]
pub struct BridgeState<L>
where
    L: Link,
{
    /// The bridge stored in the state.
    inner: BridgeStateVariant<L>,
}

impl<L> Default for BridgeState<L>
where
    L: Link,
{
    fn default() -> Self {
        panic!("bridge is not initialised!")
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

impl<L> Clone for BridgeState<L>
where
    L: Link,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<L> Eq for BridgeState<L> where L: Link {}

impl<L> BridgeState<L>
where
    L: Link,
{
    /// Creates a Bridge State from a bridge value
    pub fn from_bridge(bridge: Bridge<L>) -> Self {
        Self {
            inner: BridgeStateVariant::Value(bridge),
        }
    }

    /// Creates a Bridge State from a bridge selector
    pub fn from_bridge_selector<S>() -> Self
    where
        S: 'static + Selector + AsRef<Bridge<L>>,
    {
        Self {
            inner: BridgeStateVariant::Selector(Rc::new(|states: &BounceStates| {
                states.get_selector_value::<S>().as_ref().as_ref().clone()
            })),
        }
    }
}

pub(crate) struct BridgeSelector<L>
where
    L: Link,
{
    inner: Bridge<L>,
}

impl<L> PartialEq for BridgeSelector<L>
where
    L: Link,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<L> Eq for BridgeSelector<L> where L: Link {}

impl<L> Selector for BridgeSelector<L>
where
    L: 'static + Link,
{
    fn select(states: &BounceStates) -> Rc<Self> {
        let state = states.get_atom_value::<BridgeState<L>>();

        match state.inner {
            BridgeStateVariant::Selector(ref s) => Self { inner: s(states) },
            BridgeStateVariant::Value(ref v) => Self { inner: v.clone() },
        }
        .into()
    }
}

impl<L> Deref for BridgeSelector<L>
where
    L: Link,
{
    type Target = Bridge<L>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
