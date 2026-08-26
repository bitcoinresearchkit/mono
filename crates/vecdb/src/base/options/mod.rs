use rawdb::Database;

mod from;

use crate::Version;

/// Options for importing or creating stored vectors.
#[derive(Clone, Copy)]
pub struct ImportOptions<'a> {
    /// Database to store the vector in.
    pub db: &'a Database,
    /// Name of the vector.
    pub name: &'a str,
    /// Version for tracking data schema compatibility.
    pub version: Version,
    /// Number of stamped change files to keep for rollback support (0 to disable).
    pub saved_stamped_changes: u16,
    /// Overrides the index type's initial value capacity when set.
    pub initial_capacity: Option<usize>,
    /// Overrides the compression strategy's maximum uncompressed chunk size.
    pub max_compression_chunk_size: Option<usize>,
}

impl<'a> ImportOptions<'a> {
    pub fn new(db: &'a Database, name: &'a str, version: Version) -> Self {
        Self {
            db,
            name,
            version,
            saved_stamped_changes: 0,
            initial_capacity: None,
            max_compression_chunk_size: None,
        }
    }

    pub fn with_saved_stamped_changes(mut self, num: u16) -> Self {
        self.saved_stamped_changes = num;
        self
    }

    pub fn with_initial_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = Some(capacity);
        self
    }

    pub fn with_max_compression_chunk_size(mut self, bytes: usize) -> Self {
        self.max_compression_chunk_size = Some(bytes);
        self
    }
}
