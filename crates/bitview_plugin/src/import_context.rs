use std::path::Path;

/// Shared resources available while importing a plugin composition.
#[derive(Clone, Copy, Debug)]
pub struct ImportContext<'a> {
    data_path: &'a Path,
}

impl<'a> ImportContext<'a> {
    pub const fn new(data_path: &'a Path) -> Self {
        Self { data_path }
    }

    pub const fn data_path(self) -> &'a Path {
        self.data_path
    }
}
