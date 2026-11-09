use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{
    LazyPerSecondWindows, LazyWindowStartVec, PerBlockFullFromCumulative,
    ValuePerBlockCumulativeRolling, shapes::Windows,
};
use brk_error::Result;
use brk_types::Version;

use super::{STORAGE, Vecs, features, fees, inscription, patterns, versions, vsize};
use crate::flagged::{Classified, Totals};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        STORAGE.import(context, 10_000_000, |db, version| {
            let count = PerBlockFullFromCumulative::import(
                &db,
                "tx_count",
                version,
                &mappings.transaction_count_source(),
                mappings,
                window_starts,
            )?;
            let transfer_volume = ValuePerBlockCumulativeRolling::import(
                &db,
                "transfer_volume",
                version,
                mappings,
                window_starts,
            )?;
            let totals = Totals {
                txs: &mappings.transaction_count_source(),
                volume: transfer_volume.cumulative.sats.resolutions.height_source(),
            };
            let features =
                features::Vecs::import(&db, version, indexer, mappings, window_starts, totals)?;
            let vsize = vsize::Vecs::import(&db, version, indexer, mappings)?;
            let fees = fees::Vecs::import(&db, version, mappings, window_starts, totals)?;
            let inscription = inscription::Vecs::import(
                &db,
                version,
                indexer,
                mappings,
                window_starts,
                &fees.total_cumulative,
                totals,
            )?;
            let patterns = patterns::Vecs::import(&db, version, mappings, window_starts, totals)?;
            let nonstandard = Classified::import(
                &db,
                "is_nonstandard",
                "nonstandard_tx",
                version + Version::ONE,
                mappings,
                window_starts,
                totals,
            )?;
            let versions = versions::Vecs::import(&db, version, mappings, window_starts, totals)?;
            let per_second =
                LazyPerSecondWindows::new("tx_per_second", version + Version::TWO, &count.rolling.sum);

            Ok(Self {
                db,
                count,
                features,
                vsize,
                fees,
                inscription,
                patterns,
                nonstandard,
                versions,
                transfer_volume,
                per_second,
            })
        })
    }
}
