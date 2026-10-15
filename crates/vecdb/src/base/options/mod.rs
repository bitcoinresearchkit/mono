use rawdb::Database;

pub mod from;

use crate::Version;

/// Options for importing or creating stored vectors.
#[derive(Clone, Copy)]
pub struct ImportOptions<'a> {
    /// Database to store the vector in.
    pub(crate) db: &'a Database,
    /// Name of the vector.
    pub(crate) name: &'a str,
    /// Version for tracking data schema compatibility.
    pub(crate) version: Version,
    /// Number of stamped change files to keep for rollback support (0 to disable).
    pub(crate) saved_stamped_changes: u16,
    /// Overrides the index type's initial value capacity when set.
    pub(crate) initial_capacity: Option<usize>,
    /// Readers keep reading while updates are written in place: no reader reads updated values
    /// until written.
    pub(crate) unread_updates: bool,
}

impl<'a> ImportOptions<'a> {
    pub fn new(db: &'a Database, name: &'a str, version: Version) -> Self {
        Self {
            db,
            name,
            version,
            saved_stamped_changes: 0,
            initial_capacity: None,
            unread_updates: false,
        }
    }

    pub fn with_saved_stamped_changes(mut self, num: u16) -> Self {
        self.saved_stamped_changes = num;
        self
    }

    /// For a mutable vector whose owner serves its updated values to readers until written (an
    /// update published from memory): writing updates in place leaves its readers reading.
    ///
    /// # Safety
    /// While its updates are written, the vector and its read-only clones are read only one value
    /// at a time (`collect_one`), never a value the write changes (updated, filled, pushed or
    /// deleted), and nothing holds a reader of them.
    pub unsafe fn with_unread_updates(mut self) -> Self {
        self.unread_updates = true;
        self
    }
}
