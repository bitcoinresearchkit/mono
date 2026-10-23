use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{
    LazyPerSecondWindows, LazyWindowStartVec, PerBlockCumulativeRolling,
    PerBlockFullFromCumulative, ValuePerBlockCumulativeRolling, shapes::Windows,
};
use brk_error::Result;
use brk_types::Version;

use super::{STORAGE, Vecs, features, fees, inscription, patterns, versions, vsize};
use crate::flagged::Classified;

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 10_000_000)?;
        let version = STORAGE.schema_version();

        let count = PerBlockFullFromCumulative::import(
            &db,
            "tx_count",
            version,
            &mappings.transaction_count_source(),
            mappings,
            window_starts,
        )?;
        let all_txs = &mappings.transaction_count_source();
        let features =
            features::Vecs::import(&db, version, indexer, mappings, window_starts, all_txs)?;
        let vsize = vsize::Vecs::import(&db, version, indexer, mappings)?;
        let fees = fees::Vecs::import(&db, version, mappings, window_starts, all_txs)?;
        let inscription = inscription::Vecs::import(
            &db,
            version,
            indexer,
            mappings,
            window_starts,
            &fees.total_cumulative,
            all_txs,
        )?;
        let patterns = patterns::Vecs::import(&db, version, mappings, window_starts, all_txs)?;
        let nonstandard = Classified::import(
            &db,
            "is_nonstandard",
            "nonstandard_tx",
            version + Version::ONE,
            mappings,
            window_starts,
            all_txs,
        )?;
        let sigop_cost = PerBlockCumulativeRolling::import(
            &db,
            "block_sigop_cost",
            version,
            mappings,
            window_starts,
        )?;
        let versions = versions::Vecs::import(&db, version, mappings, window_starts, all_txs)?;
        let transfer_volume = ValuePerBlockCumulativeRolling::import(
            &db,
            "transfer_volume",
            version,
            mappings,
            window_starts,
        )?;
        let per_second =
            LazyPerSecondWindows::new("tx_per_second", version + Version::TWO, &count.rolling.sum);

        let this = Self {
            db,
            count,
            features,
            vsize,
            fees,
            inscription,
            patterns,
            nonstandard,
            sigop_cost,
            versions,
            transfer_volume,
            per_second,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
