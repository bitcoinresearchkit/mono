use crate::SAVED_CHECKPOINTS;
use crate::{
    STORAGE,
    addr::{
        AddrActivityVecs, AddrCountsVecs, AddrStateVecs, AddrVecs, AvgAmountVecs, DeltaVecs,
        ExposedAddrVecs, FundedAddrCountsVecs, NewAddrCountVecs, ReusedAddrVecs,
        TotalAddrCountVecs,
    },
    metrics::CohortMetrics,
};
use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_inputs::ByTypeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_outputs::ByTypeVecs as OutputsByTypeVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Height, Sats, StoredU64, Version};
use vecdb::{ImportOptions, ImportableVec, MutableVec, ReadableBoxedVec, ReadableCloneableVec};

use super::Vecs;

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        prices: &PriceVecs,
        inputs_by_type: &ByTypeVecs,
        outputs_by_type: &OutputsByTypeVecs,
        all_supply: &ReadableBoxedVec<Height, Sats>,
        all_utxo_count: &impl ReadableCloneableVec<Height, StoredU64>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;

        let caps = MutableVec::forced_import_with(
            ImportOptions::new(&db, "cohort_caps", Version::ONE)
                .with_saved_stamped_changes(SAVED_CHECKPOINTS),
        )?;
        let version = STORAGE.schema_version();
        let spot_price = prices.spot.cents.height.read_only_boxed_clone();

        let cohorts = CohortMetrics::forced_import(
            &db,
            version,
            mappings,
            window_starts,
            &spot_price,
            all_supply,
        )?;

        let addr_state = AddrStateVecs::forced_import(&db, version)?;

        let funded_addr_count =
            FundedAddrCountsVecs::forced_import(&db, version, mappings, window_starts)?;
        let empty_addr_count =
            AddrCountsVecs::forced_import(&db, "empty_addr_count", version, mappings)?;
        let addr_activity = AddrActivityVecs::forced_import(&db, version, mappings, window_starts)?;

        // Stored total = addr_count + empty_addr_count (global + per-type, with all derived mappings)
        let total_addr_count = TotalAddrCountVecs::forced_import(&db, version, mappings)?;

        // Per-block delta of total (global + per-type)
        let new_addr_count =
            NewAddrCountVecs::new(version, &total_addr_count, mappings, window_starts);

        // Reused address tracking (counts + per-block uses + percent).
        // `reused_*` uses the receive-side predicate (funded_txo_count > 1,
        // industry standard). `respent_*` uses the spend-side counterpart
        // (spent_txo_count > 1, strictly more restrictive).
        let reused_addr_count = ReusedAddrVecs::forced_import(
            &db,
            "reused",
            version,
            mappings,
            window_starts,
            &spot_price,
            outputs_by_type,
            inputs_by_type,
            all_supply,
        )?;
        let respent_addr_count = ReusedAddrVecs::forced_import(
            &db,
            "respent",
            version,
            mappings,
            window_starts,
            &spot_price,
            outputs_by_type,
            inputs_by_type,
            all_supply,
        )?;

        // Exposed address tracking (counts + supply) - quantum / pubkey-exposure sense
        let exposed_addr_vecs =
            ExposedAddrVecs::forced_import(&db, version, mappings, &spot_price, all_supply)?;

        // Growth rate: delta change + rate (global + per-type)
        let delta = DeltaVecs::new(version, &funded_addr_count.counts, window_starts, mappings);

        // Average amount (supply / utxo_count, supply / funded_addr_count) for `all` and per addr type.
        let avg_amount = AvgAmountVecs::forced_import(
            &db,
            version,
            mappings,
            &spot_price,
            all_supply,
            all_utxo_count,
            &funded_addr_count.counts.all.height,
        )?;

        let this = Self {
            addrs: AddrVecs {
                funded: funded_addr_count,
                empty: empty_addr_count,
                activity: addr_activity,
                total: total_addr_count,
                new: new_addr_count,
                reused: reused_addr_count,
                respent: respent_addr_count,
                exposed: exposed_addr_vecs,
                delta,
                avg_amount,
            },

            cohorts,
            addr_state,
            db,
            live: None,
            caps,
        };

        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
