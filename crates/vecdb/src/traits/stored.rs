use crate::{
    ImportableVec, ReadableCloneableVec, ReadableVec, TypedVec, VecIndex, VecValue, WritableVec,
};

/// Super trait combining all common stored vec traits.
pub trait StoredVec:
    ImportableVec + TypedVec + WritableVec<Self::I, Self::T> + ReadableCloneableVec<Self::I, Self::T>
where
    Self::I: VecIndex,
    Self::T: VecValue,
{
    /// The concrete lean read-only type returned by [`read_only_clone`](StoredVec::read_only_clone).
    type ReadOnly: TypedVec<I = Self::I, T = Self::T>
        + ReadableVec<Self::I, Self::T>
        + Clone
        + 'static;

    /// Creates a lean read-only clone that only carries fields needed for disk reads.
    ///
    /// A clone reads the written state. Between a rollback or a truncation and the next write it
    /// sees the reduced length over the old bytes: a `MutableVec`'s restored rows read the values
    /// the rollback undid, an `OverflowVec`'s may point past its reduced sidecar (reads panic), and
    /// an `OverflowVec`'s length stays above its truncated rows. Writers keep readers off the
    /// changed rows until that write.
    fn read_only_clone(&self) -> Self::ReadOnly;
}
