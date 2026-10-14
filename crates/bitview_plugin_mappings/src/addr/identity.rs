use bitview_traversable::Traversable;
use brk_types::Version;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, IndexVec, ReadableBoxedVec, ReadableCloneableVec, VecIndex, VecValue};

#[derive(Clone, Traversable)]
pub struct IdentityVecs<I, S>
where
    I: VecIndex + Formattable + Serialize + JsonSchema,
    S: VecValue,
{
    /// Zero-based type-specific output index assigned in canonical chain order.
    pub identity: IndexVec<I, I, ReadableBoxedVec<I, S>>,
}

impl<I, S> IdentityVecs<I, S>
where
    I: VecIndex + Formattable + Serialize + JsonSchema,
    S: VecValue,
{
    pub fn new(name: &str, version: Version, source: &impl ReadableCloneableVec<I, S>) -> Self {
        Self {
            identity: IndexVec::new(name, version, source.read_only_boxed_clone(), |index| index),
        }
    }
}
