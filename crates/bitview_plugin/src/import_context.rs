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

    /// Shutdown coordinator. The runtime's bootstrap holds its lock around every import, so an
    /// import must not block on outside services.
    pub const fn exit(self) -> &'a Exit {
        self.exit
    }
}
