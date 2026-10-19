use crate::{ReadWriteCompressedVec, cache::NoCache};

pub mod strategy;
pub mod r#trait;
pub mod value;

pub use strategy::*;
pub use r#trait::*;
pub use value::*;

/// Compressed vector: Pcodec over numeric values implementing [`Pco`]. An uncached read from a
/// compressed page decodes the whole page; the final partial page is stored raw.
pub type PcoVec<I, T, C = NoCache> = ReadWriteCompressedVec<I, T, PcodecStrategy<T>, C>;
