use super::{Level, VersionId};

/// Immutable version contents shared by readers.
pub struct Inner {
    pub id: VersionId,
    pub levels: Vec<Level>,
}
