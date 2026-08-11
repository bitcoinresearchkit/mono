use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{EmptyAddrData, EmptyAddrIndex, FundedAddrData, FundedAddrIndex, Height};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, BytesVec, Rw, Stamp, StorageMode, WritableVec};

/// Storage for both funded and empty address data.
#[derive(Traversable)]
pub struct AddrsDataVecs<M: StorageMode = Rw> {
    pub funded: M::Stored<BytesVec<FundedAddrIndex, FundedAddrData>>,
    pub empty: M::Stored<BytesVec<EmptyAddrIndex, EmptyAddrData>>,
}

impl AddrsDataVecs {
    /// Get minimum stamped height across funded and empty data.
    pub fn min_stamped_len(&self) -> Height {
        Height::from(self.funded.stamp())
            .incremented()
            .min(Height::from(self.empty.stamp()).incremented())
    }

    /// Rollback both funded and empty data to before the given stamp.
    pub fn rollback_before(&mut self, stamp: Stamp) -> Result<[Stamp; 2]> {
        Ok([
            self.funded.rollback_before(stamp)?,
            self.empty.rollback_before(stamp)?,
        ])
    }

    /// Reset both funded and empty data.
    pub fn reset(&mut self) -> Result<()> {
        self.funded.reset()?;
        self.empty.reset()?;
        Ok(())
    }

    /// Returns a parallel iterator over all vecs for parallel writing.
    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        vec![
            &mut self.funded as &mut dyn AnyStoredVec,
            &mut self.empty as &mut dyn AnyStoredVec,
        ]
        .into_par_iter()
    }
}
