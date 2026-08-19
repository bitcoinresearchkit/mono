use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::{Bitcoin, Height, StoredF64};
use vecdb::Exit;

use super::Vecs;
use bitview_compute::{PerBlock, PerBlockCumulativeRolling};

fn compute_rest(
    starting_height: Height,
    created: &PerBlockCumulativeRolling<StoredF64>,
    consumed: &PerBlockCumulativeRolling<StoredF64>,
    stored: &mut PerBlockCumulativeRolling<StoredF64>,
    activity: &mut PerBlock<StoredF64>,
    exit: &Exit,
) -> Result<()> {
    stored.cumulative.height.compute_subtract(
        starting_height,
        &created.cumulative.height,
        &consumed.cumulative.height,
        exit,
    )?;

    activity.height.compute_divide(
        starting_height,
        &consumed.cumulative.height,
        &created.cumulative.height,
        exit,
    )?;

    Ok(())
}

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    distribution: &bitview_plugin_distribution::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, distribution, exit)
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        distribution: &bitview_plugin_distribution::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let circulating_supply = &distribution.cohorts.supply.total.cohorts.all.sats.height;

        self.coinblocks_created.compute_cumulative_transformed(
            starting_height,
            circulating_supply,
            |value| StoredF64::from(Bitcoin::from(value)),
            exit,
        )?;

        compute_rest(
            starting_height,
            &self.coinblocks_created,
            &distribution.coinblocks_destroyed,
            &mut self.coinblocks_stored,
            &mut self.derived.liveliness,
            exit,
        )
    }
}
