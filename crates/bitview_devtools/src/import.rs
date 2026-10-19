use bitview_default::DefaultPlugins;
use bitview_plugin::ImportContext;
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tempfile::{TempDir, tempdir};
use vecdb::Budgeted;

/// A fresh offline import; the data directory lives as long as this value.
pub struct Imported {
    pub plugins: DefaultPlugins,
    pub dir: TempDir,
}

/// Imports every plugin into a temporary directory, without a node.
pub fn import() -> Result<Imported> {
    Budgeted::init_global(bitview::DEFAULT_CACHE_BUDGET)?;
    let dir = tempdir()?;
    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new(dir.path().join("blocks"), &client)?;
    let exit = Exit::new();
    let context = ImportContext::new(dir.path(), &exit);
    let plugins = DefaultPlugins::import(context, &reader)?;
    Ok(Imported { plugins, dir })
}
