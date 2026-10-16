use std::marker::PhantomData;

use crate::{
    ReadOnlyBaseVec, VecValue,
    cache::{CachePolicy, NoCache},
};

mod cached;
mod persisted;
pub mod read_only;
pub mod read_write;
pub mod strategy;

pub use read_write::*;
pub use strategy::*;

/// Lean read-only view of a raw vector (~40 bytes).
///
/// Carries only the fields needed for disk reads: region, shared length,
/// name/header metadata. No holes, no updated map, no pushed buffer,
/// no rollback state.
///
/// Created via `ReadWriteRawVec::read_only_clone`.
#[derive(Debug, Clone)]
pub struct ReadOnlyRawVec<I, T: VecValue, S, C: CachePolicy = NoCache> {
    base: ReadOnlyBaseVec<I, T>,
    cache: C::State<T>,
    _strategy: PhantomData<S>,
}
