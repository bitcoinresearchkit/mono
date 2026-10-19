pub mod bytes;
pub mod conversions;

/// Checkpoint a vector's data was last written at; rollback history is kept per stamp.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub struct Stamp(u64);
