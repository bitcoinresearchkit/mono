use brk_cohort::AddrTypeId;
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, CachedBoxedVec, Database, Rw, StorageMode, WritableVec};

use crate::{
    indexes,
    internal::{
        ColumnarPerBlock, LazyColumnSpotValuePerBlock, LazySpotValuePerBlock, WithAddrTypes,
    },
};

use super::AddrTypeToSupply;

/// Per-addr-type running supply (sats/btc/cents/usd) with an aggregated `all`.
/// Shared across predicate-based supply categories (exposed, reused, respent).
/// Sats are pushed stateful per block; cents/usd are derived post-hoc from
/// sats × spot price.
#[derive(Deref, DerefMut, Traversable)]
pub struct AddrSupplyVecs<M: StorageMode = Rw>(
    #[traversable(flatten)]
    pub  ColumnarPerBlock<
        Sats,
        AddrTypeId,
        WithAddrTypes<LazyColumnSpotValuePerBlock<AddrTypeId>, LazySpotValuePerBlock>,
        M,
    >,
);

impl AddrSupplyVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let name = format!("{name}_addr_supply");
        Ok(Self(ColumnarPerBlock::forced_import(
            db,
            &format!("{name}_sats_by_type"),
            version,
            |source| {
                WithAddrTypes::from_columnar_spot_value_source(
                    &name, version, source, indexes, spot_price,
                )
            },
        )?))
    }

    pub fn min_resume_len(&self) -> usize {
        self.height.len()
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        rayon::iter::once(self.stored_mut())
    }

    pub fn reset_height(&mut self) -> Result<()> {
        self.height.reset()?;
        Ok(())
    }

    #[inline(always)]
    pub fn push_supply(&mut self, supply: &AddrTypeToSupply) {
        self.push(supply.row());
    }
}
