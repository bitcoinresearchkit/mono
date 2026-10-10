use bitview_cohort::{CohortContext, CohortId};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, Price, PriceWithMvrv};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableVec, Rw, StorageMode};

use super::Sources;

/// Creation-price statistics of the cohort's unspent outputs.
#[derive(Traversable)]
pub struct CostBasisMetrics<M: StorageMode = Rw> {
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by satoshis.
    pub(crate) per_coin: PerCoin<M>,
}

#[derive(Traversable)]
pub struct PerCoin<M: StorageMode = Rw> {
    /// The weighted mean.
    pub(crate) avg: PriceWithMvrv<M>,
    /// Realized price: realized cap divided by supply, the weighted mean.
    realized_price: Price<LazyPerBlock<Cents>>,
}

impl CostBasisMetrics {
    pub(crate) fn import(
        db: &Database,
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
    ) -> Result<Self> {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        Ok(Self {
            per_coin: PerCoin {
                avg: PriceWithMvrv::import(
                    db,
                    &name("cost_basis_per_coin_avg"),
                    &name("mvrv"),
                    version,
                    &sources.realized_price,
                    mappings,
                )?,
                realized_price: Price::from_height_source(
                    &name("realized_price"),
                    version,
                    &sources.realized_price,
                    mappings,
                ),
            },
        })
    }

    pub(crate) fn compute(
        &mut self,
        from: Height,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.per_coin.avg.compute_ratio(from, spot, exit)
    }
}
