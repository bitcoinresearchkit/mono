use std::marker::PhantomData;

use crate::{
    ReadOnlyBaseVec, VecValue,
    cache::{CachePolicy, NoCache},
};

mod persisted;
pub mod read_only;
pub mod read_write;
pub mod strategy;

pub use read_write::*;
pub use strategy::*;

/// Read-only view of a raw vector's written values, sharing the writer's region, stored length
/// and header. Created by `StoredVec::read_only_clone`.
#[derive(Debug, Clone)]
pub struct ReadOnlyRawVec<I, T: VecValue, S, C: CachePolicy = NoCache> {
    base: ReadOnlyBaseVec<I, T>,
    cache: C::State<T>,
    _strategy: PhantomData<S>,
}
