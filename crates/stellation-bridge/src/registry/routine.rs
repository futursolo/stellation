use std::any::TypeId;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use super::Incoming;
use crate::routines::{BridgedMutation, BridgedQuery, MutationResult, QueryResult};
use crate::{BridgeError, BridgeResult};

/// The Registry Builder for Routine Registry
#[derive(Default)]
pub struct RoutineRegistryBuilder {
    query_ids: Vec<TypeId>,
}

impl fmt::Debug for RoutineRegistryBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoutineRegistryBuilder")
            .finish_non_exhaustive()
    }
}

impl RoutineRegistryBuilder {
    /// Creates a registry builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a registry.
    pub fn build(self) -> RoutineRegistry {
        RoutineRegistry {
            inner: Arc::new(self),
        }
    }

    /// Adds a mutation.
    pub fn add_mutation<T>(mut self) -> Self
    where
        T: 'static + BridgedMutation,
    {
        let type_id = TypeId::of::<T>();
        self.query_ids.push(type_id);

        self
    }

    /// Adds a query.
    pub fn add_query<T>(mut self) -> Self
    where
        T: 'static + BridgedQuery,
    {
        let type_id = TypeId::of::<T>();
        self.query_ids.push(type_id);

        self
    }
}

/// The Registry that holds available queries and mutations.
pub struct RoutineRegistry {
    inner: Arc<RoutineRegistryBuilder>,
}

impl fmt::Debug for RoutineRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoutineRegistry").finish_non_exhaustive()
    }
}

impl Clone for RoutineRegistry {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl RoutineRegistry {
    /// Creates a Builder for remote registry.
    pub fn builder() -> RoutineRegistryBuilder {
        RoutineRegistryBuilder::new()
    }

    /// The method to encode the query input for a remote link.
    pub(crate) fn encode_query_input<T>(&self, input: &T::Input) -> BridgeResult<Vec<u8>>
    where
        T: 'static + BridgedQuery,
    {
        let input = bincode::serialize(&input).map_err(BridgeError::Encoding)?;
        let type_id = TypeId::of::<T>();

        let query_index = self
            .inner
            .query_ids
            .iter()
            .enumerate()
            .find(|(_, m)| **m == type_id)
            .ok_or(BridgeError::InvalidType(type_id))?
            .0;

        let incoming = Incoming {
            query_index,
            input: &input,
        };

        bincode::serialize(&incoming).map_err(BridgeError::Encoding)
    }

    /// The method to decode the query output for a remote link.
    pub(crate) fn decode_query_output<T>(&self, output: &[u8]) -> QueryResult<T>
    where
        T: 'static + BridgedQuery,
    {
        bincode::deserialize::<std::result::Result<T, T::Error>>(output)
            .map_err(BridgeError::Encoding)
            .map_err(T::into_query_error)?
            .map(Rc::new)
    }

    /// The method to encode the mutation input for a remote link.
    pub(crate) fn encode_mutation_input<T>(&self, input: &T::Input) -> BridgeResult<Vec<u8>>
    where
        T: 'static + BridgedMutation,
    {
        let input = bincode::serialize(&input).map_err(BridgeError::Encoding)?;
        let type_id = TypeId::of::<T>();

        let query_index = self
            .inner
            .query_ids
            .iter()
            .enumerate()
            .find(|(_, m)| **m == type_id)
            .ok_or(BridgeError::InvalidType(type_id))?
            .0;

        let incoming = Incoming {
            query_index,
            input: &input,
        };

        bincode::serialize(&incoming).map_err(BridgeError::Encoding)
    }

    /// The method to decode the mutation output for a remote link.
    pub(crate) fn decode_mutation_output<T>(&self, output: &[u8]) -> MutationResult<T>
    where
        T: 'static + BridgedMutation,
    {
        bincode::deserialize::<std::result::Result<T, T::Error>>(output)
            .map_err(BridgeError::Encoding)
            .map_err(T::into_mutation_error)?
            .map(Rc::new)
    }
}
