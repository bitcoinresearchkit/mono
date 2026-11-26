#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Recent,
    Random,
    Vec,
}

/// When a store compacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compaction {
    /// As it is written: it is read meanwhile.
    Eager,
    /// Deferred from its opening until the database's `resume_compaction`: for a store written
    /// in bulk that is read little meanwhile.
    Deferred,
}
