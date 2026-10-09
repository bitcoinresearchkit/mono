use bitview_default::DefaultPlugins;
use bitview_plugin::ImportContext;
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_price::HasPrice;
use bitview_plugin_profitability::Vecs as Profitability;
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tempfile::{TempDir, tempdir};
use vecdb::{Budgeted, ReadableCloneableVec, Rw, StorageMode};

/// Every plugin in the repository, composed as the optional-plugin examples do.
#[derive(PluginSet, Traversable)]
pub struct AllPlugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    #[plugin_set(flatten)]
    pub(crate) defaults: DefaultPlugins<M>,
    #[traversable(flatten)]
    profitability: Profitability<M>,
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
    let window_starts = defaults.blocks().lookback.window_starts();
    let prices = defaults.price().spot.cents.height.read_only_boxed_clone();
    let profitability =
        Profitability::import(context, defaults.mappings(), &window_starts, &prices)?;

    Ok(Imported {
        plugins: AllPlugins {
            defaults,
            profitability,
        },
        dir,
    })
}
