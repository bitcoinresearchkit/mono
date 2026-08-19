use crate::Vecs;

/// Provides access to the constants plugin.
pub trait HasConstants {
    fn constants(&self) -> &Vecs;
}
