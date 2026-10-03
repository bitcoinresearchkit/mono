use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::WeightToVSize;
use bitview_vecs::{LazyPerTxDistributionTransformed, TxDerivedDistribution};
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::Vecs;

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let weight = TxDerivedDistribution::forced_import(db, "tx_weight", version, mappings)?;

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

        Ok(Vecs { vsize, weight })
    }
}
