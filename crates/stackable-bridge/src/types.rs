use std::error::Error;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::error::BridgeError;

#[cold]
fn panic_network_error(e: BridgeError) -> ! {
    panic!("failed to communicate with server: {:?}", e);
}

pub trait BridgedQuery: Serialize + for<'de> Deserialize<'de> + PartialEq {
    type Input: 'static + Serialize + for<'de> Deserialize<'de> + Hash + Eq + Clone;
    type Error: 'static + Serialize + for<'de> Deserialize<'de> + Error + PartialEq + Clone;

    #[cold]
    fn into_query_error(e: BridgeError) -> Self::Error {
        panic_network_error(e);
    }
}

pub type QueryResult<T> = std::result::Result<Rc<T>, <T as BridgedQuery>::Error>;

pub trait BridgedMutation: Serialize + for<'de> Deserialize<'de> + PartialEq {
    type Input: 'static + Serialize + for<'de> Deserialize<'de>;
    type Error: 'static + Serialize + for<'de> Deserialize<'de> + Error + PartialEq + Clone;

    #[cold]
    fn into_mutation_error(e: BridgeError) -> Self::Error {
        panic_network_error(e);
    }
}

pub type MutationResult<T> = std::result::Result<Rc<T>, <T as BridgedMutation>::Error>;

/// A placeholder type until never type lands in std.
#[derive(thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[error("this never happens")]
pub struct Never(PhantomData<()>);
