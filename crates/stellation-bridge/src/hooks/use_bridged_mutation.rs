use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_mutation, MutationState, UseMutationHandle};
use bounce::BounceStates;
use yew::prelude::*;

use crate::links::Link;
use crate::routines::{BridgedMutation, MutationResult};
use crate::state::BridgeSelector;

/// Bridged Mutation State
#[derive(Debug, PartialEq)]
pub enum BridgedMutationState<T>
where
    T: BridgedMutation + 'static,
{
    /// The mutation has not started yet.
    Idle,
    /// The mutation is loading.
    Loading,
    /// The mutation has completed.
    Completed {
        /// Result of the completed mutation.
        result: MutationResult<T>,
    },
    /// A previous mutation has completed and a new mutation is currently loading.
    Refreshing {
        /// Result of last completed mutation.
        last_result: MutationResult<T>,
    },
}

impl<T> Clone for BridgedMutationState<T>
where
    T: BridgedMutation + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Idle => Self::Idle,
            Self::Loading => Self::Loading,
            Self::Completed { result } => Self::Completed {
                result: result.clone(),
            },
            Self::Refreshing { last_result } => Self::Refreshing {
                last_result: last_result.clone(),
            },
        }
    }
}

impl<T> PartialEq<&BridgedMutationState<T>> for BridgedMutationState<T>
where
    T: BridgedMutation + 'static,
{
    fn eq(&self, other: &&BridgedMutationState<T>) -> bool {
        self == *other
    }
}

impl<T> PartialEq<BridgedMutationState<T>> for &'_ BridgedMutationState<T>
where
    T: BridgedMutation + 'static,
{
    fn eq(&self, other: &BridgedMutationState<T>) -> bool {
        *self == other
    }
}

struct BridgedMutationInner<M, L>
where
    M: BridgedMutation,
{
    inner: MutationResult<M>,
    _marker: PhantomData<L>,
}

impl<M, L> PartialEq for BridgedMutationInner<M, L>
where
    M: BridgedMutation,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[async_trait(?Send)]
impl<M, L> bounce::query::Mutation for BridgedMutationInner<M, L>
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
    inner: UseMutationHandle<BridgedMutationInner<T, L>>,
    state: Rc<BridgedMutationState<T>>,
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

    /// Returns the state of current mutation.
    pub fn state(&self) -> &BridgedMutationState<T> {
        self.state.as_ref()
    }

    /// Returns the result of last finished mutation (if any).
    ///
    /// - `None` indicates that a mutation is currently loading or has yet to start(idling).
    /// - `Some(Ok(m))` indicates that the last mutation is successful and the content is stored in
    ///   `m`.
    /// - `Some(Err(e))` indicates that the last mutation has failed and the error is stored in `e`.
    pub fn result(&self) -> Option<&MutationResult<T>> {
        match self.state() {
            BridgedMutationState::Idle | BridgedMutationState::Loading => None,
            BridgedMutationState::Completed { result }
            | BridgedMutationState::Refreshing {
                last_result: result,
            } => Some(result),
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
            .field("state", &self.state())
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
            state: self.state.clone(),
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
    let handle = use_mutation::<BridgedMutationInner<T, L>>();
    let state = use_memo(
        |state| match state {
            MutationState::Idle => BridgedMutationState::Idle,
            MutationState::Loading => BridgedMutationState::Loading,
            MutationState::Completed { result } => BridgedMutationState::Completed {
                result: result
                    .as_ref()
                    .map_err(|e| e.clone())
                    .and_then(|m| m.inner.clone()),
            },
            MutationState::Refreshing { last_result } => BridgedMutationState::Refreshing {
                last_result: last_result
                    .as_ref()
                    .map_err(|e| e.clone())
                    .and_then(|m| m.inner.clone()),
            },
        },
        handle.state().clone(),
    );

    UseBridgedMutationHandle {
        inner: handle,
        state,
    }
}
