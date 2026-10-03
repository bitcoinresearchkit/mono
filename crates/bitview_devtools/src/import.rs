use bitview_default::DefaultPlugins;
use bitview_plugin::ImportContext;
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_distribution_aggregated::HasDistributionAggregated;
use bitview_plugin_distribution_entry::Vecs as DistributionEntry;
use bitview_plugin_distribution_profitability::Vecs as DistributionProfitability;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_price::HasPrice;
use bitview_runtime::PluginSet;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use tempfile::{TempDir, tempdir};
use vecdb::{Budgeted, ReadableCloneableVec, Rw, StorageMode};

/// Every plugin in the repository, composed as the optional-plugin examples do.
#[derive(PluginSet, Traversable)]
pub struct AllPlugins<M: StorageMode = Rw> {
    #[traversable(flatten)]
    #[plugin_set(flatten)]
    pub defaults: DefaultPlugins<M>,
    pub distribution_entry: DistributionEntry<M>,
    #[traversable(flatten)]
    pub distribution_profitability: DistributionProfitability<M>,
}

/// A fresh offline import; the data directory lives as long as this value.
pub struct Imported {
    pub plugins: AllPlugins,
    pub dir: TempDir,
}

/// Imports every plugin into a temporary directory, without a node (like bindgen).
pub fn import() -> Result<Imported> {
    Budgeted::init_global(bitview_plugin::DEFAULT_CACHE_BUDGET)?;
    let dir = tempdir()?;
    let client = Client::new("http://127.0.0.1:1", Auth::None)?;
    let reader = Reader::new_without_rlimit(dir.path().join("blocks"), &client);
    let context = ImportContext::new(dir.path());

    let defaults = DefaultPlugins::import(context, &reader)?;
    let window_starts = defaults.blocks().lookback.window_starts();
    let prices = defaults.price().spot.cents.height.read_only_boxed_clone();
    let distribution_entry = DistributionEntry::import(
        context,
        defaults.mappings(),
        &window_starts,
        &prices,
        defaults.distribution_aggregated().all_supply(),
    )?;
    let distribution_profitability =
        DistributionProfitability::import(context, defaults.mappings(), &window_starts, &prices)?;

    Ok(Imported {
        plugins: AllPlugins {
            defaults,
            distribution_entry,
            distribution_profitability,
        },
        dir,
    })
}
