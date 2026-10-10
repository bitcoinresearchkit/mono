use bitview_default::DefaultPlugins;
use bitview_plugin::ImportContext;
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tempfile::{TempDir, tempdir};
use vecdb::{Budgeted, Rw, StorageMode};

/// Every plugin in the repository: the defaults plus any optional plugin (none today).
#[derive(PluginSet, Traversable)]
pub struct AllPlugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    #[plugin_set(flatten)]
    pub(crate) defaults: DefaultPlugins<M>,
}

/// A fresh offline import; the data directory lives as long as this value.
pub struct Imported {
    pub plugins: AllPlugins,
    pub(crate) dir: TempDir,
}

/// Imports every plugin into a temporary directory, without a node (like bindgen).
pub fn import() -> Result<Imported> {
    Budgeted::init_global(bitview_plugin::DEFAULT_CACHE_BUDGET)?;
    let dir = tempdir()?;
    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new_without_rlimit(dir.path().join("blocks"), &client);
    let exit = Exit::new();
    let context = ImportContext::new(dir.path(), &exit);

    let defaults = DefaultPlugins::import(context, &reader)?;

    Ok(Imported {
        plugins: AllPlugins { defaults },
        dir,
    })
}
