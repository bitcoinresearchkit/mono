use brk_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, LazyVec, VecIndex, VecValue};

#[derive(Clone, Traversable)]
pub struct IdentityVecs<I, S>
where
    I: VecIndex + Formattable + Serialize + JsonSchema,
    S: VecValue,
{
    /// Zero-based type-specific output index assigned in canonical chain order.
    pub identity: LazyVec<I, I, I, S>,
}
