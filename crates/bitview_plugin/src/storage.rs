use std::{
    ffi::OsStr,
    fmt,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use brk_error::{Error, Result};
use brk_exit::Exit;
use tracing::warn;
use vecdb::{Database, PAGE_SIZE, Version};

use crate::{ImportContext, PLUGIN_DATA_DIR, PluginId};

/// Stable identity and root storage schema for one plugin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginStorage {
    id: PluginId,
    schema_version: Version,
    mainnet_only: bool,
}

impl PluginStorage {
    pub const fn new(id: PluginId, schema_version: Version) -> Self {
        Self {
            id,
            schema_version,
            mainnet_only: false,
        }
    }

    /// The plugin's source data exists only on mainnet.
    pub const fn mainnet_only(mut self) -> Self {
        self.mainnet_only = true;
        self
    }

    pub fn available(self) -> bool {
        !self.mainnet_only || brk_types::chain().is_mainnet()
    }

    pub const fn id(self) -> PluginId {
        self.id
    }

    pub const fn schema_version(self) -> Version {
        self.schema_version
    }

    pub fn plugins_path(context: ImportContext<'_>) -> PathBuf {
        context.data_path().join(PLUGIN_DATA_DIR)
    }

    pub fn path(self, context: ImportContext<'_>) -> PathBuf {
        Self::plugins_path(context).join(self.id.as_str())
    }

    /// Removes every entry of the plugins' folder that `keep` doesn't name, saying so with
    /// `reason` before each. Under the exit lock, while no plugin it removes is open.
    pub fn remove_plugin_data(
        context: ImportContext<'_>,
        keep: impl Fn(&OsStr) -> bool,
        reason: &str,
    ) -> Result<()> {
        for entry in fs::read_dir(Self::plugins_path(context))? {
            let entry = entry?;
            if keep(&entry.file_name()) {
                continue;
            }
            let path = entry.path();
            remove_data(&path, "Removing", format_args!("{reason}"), || {
                if entry.file_type()?.is_dir() {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_file(&path)
                }
            })?;
        }
        Ok(())
    }

    /// Rebuilds the whole owner once after a classified data failure. The failed import must
    /// own all of its database handles: they are dropped before this removes the directory.
    /// Locks and other operational errors propagate without deletion. Bootstrap holds the
    /// shutdown lock throughout. The failure and path are logged before any removal.
    pub fn import_with_recovery<T>(
        self,
        context: ImportContext<'_>,
        import: impl Fn() -> Result<T>,
    ) -> Result<T> {
        match import() {
            Err(error) if !error.is_lock_error() && error.is_data_error() => {
                let path = self.path(context);
                match remove_data(
                    &path,
                    "Rebuilding",
                    format_args!("import failed: {error}"),
                    || fs::remove_dir_all(&path),
                ) {
                    Ok(()) => {}
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                import()
            }
            result => result,
        }
    }

    pub fn open_database(
        self,
        context: ImportContext<'_>,
        page_multiplier: usize,
    ) -> Result<Database> {
        let path = self.path(context);
        (|| {
            let db = Database::open(&path)?;
            db.set_min_len(PAGE_SIZE * page_multiplier)?;
            Ok(db)
        })()
        .map_err(|error: Error| error.opening_storage(&path))
    }

    pub fn finalize_database(self, db: &Database) -> Result<()> {
        db.retain_accessed_regions()?;
        db.compact()?;
        Ok(())
    }
}

fn remove_data(
    path: &Path,
    action: &str,
    reason: fmt::Arguments<'_>,
    remove: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    debug_assert!(Exit::is_locked(), "plugin data removal outside the exit lock");
    warn!("{action} plugin data at {}: {reason}", path.display());
    remove().map_err(|error| {
        io::Error::new(error.kind(), format!("removing {}: {error}", path.display()))
    })
}
