use brk_error::Result;

use bitview_compute::{LazyPerTxDistributionTransformed, TxDerivedDistribution, WeightToVSize};
use bitview_plugin_indexer::Indexer;
use brk_types::Version;
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::Vecs;

pub fn forced_import(
    db: &Database,
    version: Version,
    indexer: &Indexer,
    indexes: &bitview_plugin_indexes::Vecs,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, indexer, indexes)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        let weight = TxDerivedDistribution::forced_import(db, "tx_weight", version, indexes)?;

        let tx_index_to_vsize = LazyVec::transformed::<WeightToVSize>(
            "tx_vsize",
            version,
            indexer.vecs().transactions.weight.read_only_boxed_clone(),
        );

        let vsize = LazyPerTxDistributionTransformed::new::<WeightToVSize>(
            "tx_vsize",
            version,
            tx_index_to_vsize,
            &weight,
        );

        Ok(Self { vsize, weight })
    }
}
