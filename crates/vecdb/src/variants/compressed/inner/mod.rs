use std::{marker::PhantomData, sync::Arc};

use parking_lot::RwLock;

use crate::{
    ReadOnlyBaseVec, VecValue,
    cache::{CachePolicy, NoCache},
};

pub mod decoder;
pub mod encoded_chunk;
pub mod page;
pub mod pages;
mod persisted;
pub mod read_only;
pub mod read_write;
pub mod strategy;

pub use decoder::PageDecoder;
pub use encoded_chunk::*;
pub use page::*;
pub use pages::*;
pub use read_write::*;
pub use strategy::*;

/// Read-only view of a compressed vector's written values, sharing the writer's region, page
/// table, stored length and header. Created by `StoredVec::read_only_clone`.
#[derive(Debug, Clone)]
pub struct ReadOnlyCompressedVec<I, T: VecValue, S, C: CachePolicy = NoCache> {
    base: ReadOnlyBaseVec<I, T>,
    cache: C::State<T>,
    pages: Arc<RwLock<Pages>>,
    _strategy: PhantomData<S>,
}
