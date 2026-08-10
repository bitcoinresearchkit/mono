use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{StoredF32, StoredF64, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Exit, Rw, StorageMode};

use crate::internal::{ColumnarRollingWindows, Identity, LazyPerBlock};

use crate::{
    distribution::{
        metrics::ImportConfig,
        state::{CohortState, CostBasisOps, RealizedOps},
    },
    price,
};

use super::ActivityCore;

#[derive(Deref, DerefMut, Traversable)]
pub struct ActivityFull<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub inner: ActivityCore<M>,

    pub coinyears_destroyed: LazyPerBlock<StoredF64, StoredF64>,

    pub dormancy: ColumnarRollingWindows<StoredF32, M>,
}

impl ActivityFull {
    pub(crate) fn forced_import(cfg: &ImportConfig) -> Result<Self> {
        let v1 = Version::ONE;
        let inner = ActivityCore::forced_import(cfg)?;

        let coinyears_destroyed = LazyPerBlock::from_height_source::<Identity<StoredF64>, _>(
            &cfg.name("coinyears_destroyed"),
            cfg.version + v1,
            inner.coindays_destroyed.sum._1y.height.clone(),
            cfg.indexes,
        );

        let dormancy = ColumnarRollingWindows::forced_import(
            cfg.db,
            &cfg.name("dormancy"),
            cfg.version + v1,
            cfg.indexes,
        )?;

        Ok(Self {
            inner,
            coinyears_destroyed,
            dormancy,
        })
    }

    pub(crate) fn full_min_len(&self) -> usize {
        self.inner.min_len()
    }

    #[inline(always)]
    pub(crate) fn full_push_state(
        &mut self,
        state: &CohortState<impl RealizedOps, impl CostBasisOps>,
    ) {
        self.inner.push_state(state);
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.inner.collect_vecs_mut();
        vecs.push(self.dormancy.stored_mut());
        vecs
    }

    pub(crate) fn compute_from_stateful(
        &mut self,
        starting_lengths: &Lengths,
        others: &[&ActivityCore],
        exit: &Exit,
    ) -> Result<()> {
        self.inner
            .compute_from_stateful(starting_lengths, others, exit)
    }

    pub(crate) fn compute_rest_part1(
        &mut self,
        prices: &price::Vecs,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        self.inner
            .compute_rest_part1(prices, starting_lengths, exit)?;

        let Self {
            inner, dormancy, ..
        } = self;
        let cdd_sums = &inner.coindays_destroyed.sum;
        let transfer_volume_sums = &inner.minimal.transfer_volume.sum.0;
        dormancy.compute_columns2(
            starting_lengths.height,
            |window| &window.select(cdd_sums).height,
            |window| &window.select(transfer_volume_sums).btc.height,
            |_, rolling_cdd, rolling_btc| {
                let btc = f64::from(rolling_btc);
                if btc == 0.0 {
                    StoredF32::from(0.0f32)
                } else {
                    StoredF32::from((f64::from(rolling_cdd) / btc) as f32)
                }
            },
            exit,
        )?;

        Ok(())
    }
}
