use super::Level;

/// Immutable version contents shared by readers.
pub struct Inner {
    pub id: u64,
    pub levels: Vec<Level>,
}
