use bitview_plugin_distribution_age::Vecs as AgeVecs;
use bitview_plugin_distribution_aggregated::Vecs as AggregatedVecs;
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, BoundedRatio, StoredF64};

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        distribution_age: &AgeVecs,
        distribution_aggregated: &AggregatedVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let circulating_supply = &distribution_aggregated.cohorts.all.supply.total.sats.height;

        self.coinblocks_created.compute_cumulative_transformed(
            starting_height,
            circulating_supply,
            |value| StoredF64::from(Bitcoin::from(value)),
            exit,
        )?;

        self.coinblocks_stored.cumulative.height.compute_subtract(
            starting_height,
            &self.coinblocks_created.cumulative.height,
            &distribution_age.coinblocks_destroyed.cumulative.height,
            exit,
        )?;

        self.derived.liveliness_source.height.compute_transform2(
            starting_height,
            &distribution_age.coinblocks_destroyed.cumulative.height,
            &self.coinblocks_created.cumulative.height,
            |(h, destroyed, created, ..)| {
                (
                    h,
                    BoundedRatio::from(f64::from(destroyed) / f64::from(created)),
                )
            },
            exit,
        )?;

        Ok(())
    }
}
