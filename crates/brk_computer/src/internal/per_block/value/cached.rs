use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{
    AnyVec, Database, Exit, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecIndex, VecValue,
};

use crate::{
    indexes,
    internal::{
        CachedPerBlock, CentsUnsignedToDollars, LazyPerBlock, PerBlock, SatsToBitcoin,
        ValuePerBlockCumulative,
    },
};

/// Stored cumulative value whose sats source owns a pinned in-memory cache.
#[derive(Traversable)]
pub struct CachedValuePerBlock<M: StorageMode = Rw> {
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    pub sats: CachedPerBlock<Sats, M>,
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: PerBlock<Cents, M>,
}

impl CachedValuePerBlock {
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let sats = CachedPerBlock::forced_import(db, &format!("{name}_sats"), version, indexes)?;
        let btc = LazyPerBlock::from_cached_computed::<SatsToBitcoin>(
            name,
            version,
            sats.height.read_only_boxed_clone(),
            &sats,
        );

        let cents = PerBlock::forced_import(db, &format!("{name}_cents"), version, indexes)?;
        let usd = LazyPerBlock::from_computed::<CentsUnsignedToDollars>(
            &format!("{name}_usd"),
            version,
            cents.height.read_only_boxed_clone(),
            &cents,
        );

        Ok(Self {
            btc,
            sats,
            usd,
            cents,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_sats_from_indexes<A, B>(
        &mut self,
        max_from: Height,
        first_indexes: &impl ReadableVec<Height, A>,
        indexes_count: &impl ReadableVec<Height, B>,
        source: &impl ReadableVec<A, Sats>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        if max_from.to_usize() < self.sats.height.len() {
            self.sats.height.invalidate();
        }

        ValuePerBlockCumulative::compute_sats_height_from_indexes(
            &mut self.sats.height.inner,
            max_from,
            first_indexes,
            indexes_count,
            source,
            |_| true,
            exit,
        )
    }
}
