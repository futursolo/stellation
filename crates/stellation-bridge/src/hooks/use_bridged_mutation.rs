use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_mutation, UseMutationHandle};
use bounce::BounceStates;
use yew::prelude::*;

use crate::links::Link;
use crate::routines::{BridgedMutation, MutationResult};
use crate::state::BridgeSelector;

struct MutationState<M, L>
where
    M: BridgedMutation,
{
    inner: MutationResult<M>,
    _marker: PhantomData<L>,
}

impl<M, L> PartialEq for MutationState<M, L>
where
    M: BridgedMutation,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[async_trait(?Send)]
impl<M, L> bounce::query::Mutation for MutationState<M, L>
where
    M: 'static + BridgedMutation,
    L: 'static + Link,
{
    type Error = M::Error;
    type Input = M::Input;

    async fn run(
        states: &BounceStates,
        input: Rc<M::Input>,
    ) -> bounce::query::MutationResult<Self> {
        let bridge = states.get_selector_value::<BridgeSelector<L>>();
        let link = bridge.link();

        Ok(Self {
            inner: link.resolve_mutation::<M>(&input).await,
            _marker: PhantomData,
        }
        .into())
    }
}

/// A handle returned by [`use_bridged_mutation`].
///
/// This can be used to access the result or start the mutation.
pub struct UseBridgedMutationHandle<T, L>
where
    T: BridgedMutation + 'static,
    L: 'static + Link,
{
    inner: UseMutationHandle<MutationState<T, L>>,
}

impl<T, L> UseBridgedMutationHandle<T, L>
where
    T: BridgedMutation + 'static,
    L: 'static + Link,
{
    /// Runs a mutation with input.
    pub async fn run(&self, input: impl Into<Rc<T::Input>>) -> MutationResult<T> {
        self.inner.run(input).await?.inner.clone()
    }

    /// Returns the result of last finished mutation (if any).
    ///
    /// - `None` indicates that a mutation is currently loading or has yet to start(idling).
    /// - `Some(Ok(m))` indicates that the last mutation is successful and the content is stored in
    ///   `m`.
    /// - `Some(Err(e))` indicates that the last mutation has failed and the error is stored in `e`.
    pub fn result(&self) -> Option<&MutationResult<T>> {
        match self.inner.result()? {
            Ok(m) => Some(&m.inner),
            Err(_) => panic!("this can never happen!"),
        }
    }
}

impl<T, L> fmt::Debug for UseBridgedMutationHandle<T, L>
where
    T: BridgedMutation + fmt::Debug + 'static,
    L: 'static + Link,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseBridgedMutationHandle")
            .field("state", &self.result())
            .finish()
    }
}

impl<T, L> Clone for UseBridgedMutationHandle<T, L>
where
    T: BridgedMutation + 'static,
    L: 'static + Link,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Bridges a mutation.
#[hook]
pub fn use_bridged_mutation<T, L>() -> UseBridgedMutationHandle<T, L>
where
    T: 'static + BridgedMutation,
    L: 'static + Link,
{
    let handle = use_mutation::<MutationState<T, L>>();

    UseBridgedMutationHandle { inner: handle }
}
