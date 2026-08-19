use crate::Vecs;

/// Provides access to the investing plugin.
pub trait HasInvesting {
    fn investing(&self) -> &Vecs;
}
