use std::error::Error;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

pub trait Query: Serialize + for<'de> Deserialize<'de> + PartialEq {
    type Input: 'static + Serialize + for<'de> Deserialize<'de> + Hash + Eq;
    type Error: 'static + Serialize + for<'de> Deserialize<'de> + Error + PartialEq + Clone;
}

pub type QueryResult<T> = std::result::Result<Rc<T>, <T as Query>::Error>;

pub trait Mutation: Serialize + for<'de> Deserialize<'de> + PartialEq {
    type Input: 'static + Serialize + for<'de> Deserialize<'de>;
    type Error: 'static + Serialize + for<'de> Deserialize<'de> + Error + PartialEq + Clone;
}

pub type MutationResult<T> = std::result::Result<Rc<T>, <T as Mutation>::Error>;

/// A placeholder type until never type lands in std.
#[derive(thiserror::Error, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[error("this never happens")]
pub struct Never(PhantomData<()>);
