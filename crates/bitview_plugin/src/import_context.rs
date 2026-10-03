use std::path::Path;

use brk_exit::Exit;

/// Shared resources available while importing a plugin composition.
#[derive(Clone, Copy)]
pub struct ImportContext<'a> {
    data_path: &'a Path,
    exit: &'a Exit,
}

impl<'a> ImportContext<'a> {
    pub const fn new(data_path: &'a Path, exit: &'a Exit) -> Self {
        Self { data_path, exit }
    }

    pub(crate) const fn data_path(self) -> &'a Path {
        self.data_path
    }

    /// Shutdown coordinator; hold its lock around import-time writes that are not safe to
    /// interrupt (e.g. the indexer's startup rollback or reset).
    pub const fn exit(self) -> &'a Exit {
        self.exit
    }
}
