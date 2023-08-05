//! Hooks used to resolve requests.

mod use_bridged_mutation;
mod use_bridged_query;
mod use_bridged_query_value;

pub use use_bridged_mutation::{
    use_bridged_mutation, BridgedMutationState, UseBridgedMutationHandle,
};
pub use use_bridged_query::{use_bridged_query, BridgedQueryState, UseBridgedQueryHandle};
pub use use_bridged_query_value::{
    use_bridged_query_value, BridgedQueryValueState, UseBridgedQueryValueHandle,
};
