use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_indexer::Indexer;
use bitview_primitives::BoundedRatio;
use brk_error::Result;
use brk_exit::Exit;
use vecdb::ComputeVec;

use super::{CoinblocksCreated, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        age: &AgeVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.compute_start().height;

        let CoinblocksCreated { block, running, .. } = &mut self.coinblocks_created;
        running.compute_cumulative_transformed(starting_height, block, |value| value, exit)?;

        self.coinblocks_stored.cumulative.height.compute_subtract(
            starting_height,
            &self.coinblocks_created.cumulative.height,
            &age.coinblocks_destroyed.cumulative.height,
            exit,
        )?;

        self.derived.liveliness_source.height.compute_transform2(
            starting_height,
            &age.coinblocks_destroyed.cumulative.height,
            &self.coinblocks_created.cumulative.height,
            |(_, destroyed, created, ..)| {
                BoundedRatio::from(f64::from(destroyed) / f64::from(created))
            },
            exit,
        )?;

        Ok(())
    }
}
