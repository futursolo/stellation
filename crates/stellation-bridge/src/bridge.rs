use std::fmt;
use std::rc::Rc;

use yew::prelude::*;
use yew::suspense::SuspensionResult;

use crate::hooks::{
    use_bridged_mutation, use_bridged_query, use_bridged_query_value, UseBridgedMutationHandle,
    UseBridgedQueryHandle, UseBridgedQueryValueHandle,
};
use crate::links::Link;
use crate::routines::{BridgedMutation, BridgedQuery};

/// The Bridge.
pub struct Bridge<L> {
    pub(crate) link: L,
}

impl<L> Clone for Bridge<L>
where
    L: Clone,
{
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}

impl<L> fmt::Debug for Bridge<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bridge").finish_non_exhaustive()
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
        Self { link }
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

    /// Bridges a query as value.
    ///
    /// # Note
    ///
    /// This hook does not suspend the component and the data is not fetched during SSR.
    /// If this hook is used in SSR, this hook will remain as loading state.
    pub fn use_query_value<T>(
        input: Rc<T::Input>,
    ) -> impl Hook<Output = UseBridgedQueryValueHandle<T, L>>
    where
        T: 'static + BridgedQuery,
        L: 'static,
    {
        use_bridged_query_value(input)
    }
}
