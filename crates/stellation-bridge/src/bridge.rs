use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bounce::{BounceStates, Selector};
use yew::prelude::*;
use yew::suspense::SuspensionResult;

use crate::hooks::{
    use_bridged_mutation, use_bridged_query, UseBridgedMutationHandle, UseBridgedQueryHandle,
};
use crate::links::Link;
use crate::routines::{BridgedMutation, BridgedQuery};

pub(super) type ReadToken = Rc<dyn Fn(&BounceStates) -> Rc<dyn AsRef<str>>>;

/// The Bridge.
pub struct Bridge<L> {
    id: usize,
    pub(crate) link: L,
    read_token: Option<ReadToken>,
}

impl<L> Clone for Bridge<L>
where
    L: Clone,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            link: self.link.clone(),
            read_token: self.read_token.clone(),
        }
    }
}

impl<L> fmt::Debug for Bridge<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bridge")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl<L> PartialEq for Bridge<L>
where
    L: Link,
{
    fn eq(&self, other: &Self) -> bool {
        self.link == other.link
    }
}
impl<L> Eq for Bridge<L> where L: Link {}

impl<L> Bridge<L>
where
    L: Link,
{
    /// Creates a new Bridge.
    pub fn new(link: L) -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);
        let id = ID.fetch_add(1, Ordering::AcqRel);

        Self {
            id,
            link,
            read_token: None,
        }
    }

    pub(crate) fn read_token(&self, states: &BounceStates) -> Option<Rc<dyn AsRef<str>>> {
        self.read_token.as_ref().map(|m| m(states))
    }

    /// Selects the token from a bounce state.
    pub fn with_token_selector<T>(mut self) -> Self
    where
        T: 'static + Selector + AsRef<str>,
    {
        let read_token = Rc::new(move |states: &BounceStates| {
            let state = states.get_selector_value::<T>();

            state as Rc<dyn AsRef<str>>
        }) as ReadToken;

        self.read_token = Some(read_token);

        self
    }

    /// Returns the link used by current instance.
    pub fn link(&self) -> &L {
        &self.link
    }

    /// Bridges a mutation.
    pub fn use_mutation<T>() -> impl Hook<Output = UseBridgedMutationHandle<T, L>>
    where
        T: 'static + BridgedMutation,
        L: 'static,
    {
        use_bridged_mutation()
    }

    /// Bridges a query.
    pub fn use_query<T>(
        input: Rc<T::Input>,
    ) -> impl Hook<Output = SuspensionResult<UseBridgedQueryHandle<T, L>>>
    where
        T: 'static + BridgedQuery,
        L: 'static,
    {
        use_bridged_query(input)
    }
}
