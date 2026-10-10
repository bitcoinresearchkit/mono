use bitview_cohort::{AddressType, AmountRange};
use bitview_collections::Windows;
use bitview_distribution::RealizedCaps;
use bitview_plugin::ImportContext as PluginImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use vecdb::ReadableCloneableVec;

use crate::{
    SAVED_CHECKPOINTS, STORAGE, Vecs,
    addr::{AddrStateVecs, AddressVecs, ImportContext},
    balance::BalanceVecs,
};

impl Vecs {
    pub fn import(
        context: PluginImportContext<'_>,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &PriceVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let caps = RealizedCaps::import(&db, SAVED_CHECKPOINTS)?;
        let version = STORAGE.schema_version();
        let spot = prices.spot.cents.height.read_only_boxed_clone();
        let ctx = ImportContext {
            db: &db,
            version,
            mappings,
            windows,
            spot: &spot,
        };
        let all = Box::new(AddressVecs::import(&ctx, "")?);
        let types = Box::new(AddressType::try_from_fn(|id| {
            AddressVecs::import(&ctx, &format!("{}_", id.key()))
        })?);
        let address_supply = all.supply().read_only_boxed_clone();
        let balances = Box::new(AmountRange::try_new(|cohort| {
            BalanceVecs::import(&ctx, cohort, &address_supply)
        })?);
        let state = AddrStateVecs::import(&db, version)?;
        let this = Self {
            db,
            live: None,
            caps,
            all,
            types,
            balances,
            state,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
