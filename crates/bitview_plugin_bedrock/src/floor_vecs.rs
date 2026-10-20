use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, IndexSources, LazyFiatPerBlock, LazySpotValuePerBlock, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode, WritableVec};

use super::{FloorBands, Holding};

/// A mode's holdings by the band of its floors spot sat in when they were created.
#[derive(Traversable)]
pub struct FloorVecs<M: StorageMode = Rw> {
    /// Supply created since the second halving (block 420,000) while spot sat in this band of
    /// the mode's floors at its creation block, less what has been spent since.
    pub floor_supply: FloorBands<LazySpotValuePerBlock>,
    /// The capital of that supply: each unspent output at its creation price (the band's
    /// realized cap).
    pub floor_capital: FloorBands<LazyFiatPerBlock<Cents>>,
    #[traversable(hidden)]
    supply_stored: FloorBands<CachedSeries<Height, Sats, M>>,
    #[traversable(hidden)]
    capital_stored: FloorBands<CachedSeries<Height, Cents, M>>,
}

impl FloorVecs {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &IndexSources,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let supply_stored = FloorBands::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{name}_floor_supply_{}_sats", id.name()),
                version,
            )
        })?;
        let capital_stored = FloorBands::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{name}_floor_capital_{}_cents", id.name()),
                version,
            )
        })?;
        Ok(Self {
            floor_supply: FloorBands::from_fn(|id| {
                LazySpotValuePerBlock::from_sats_source(
                    &format!("{name}_floor_supply_{}", id.name()),
                    version,
                    supply_stored.select(id),
                    mappings,
                    spot_price,
                )
            }),
            floor_capital: FloorBands::from_fn(|id| {
                LazyFiatPerBlock::from_cents_source(
                    &format!("{name}_floor_capital_{}", id.name()),
                    version,
                    capital_stored.select(id),
                    mappings,
                )
            }),
            supply_stored,
            capital_stored,
        })
    }

    pub(crate) fn push(&mut self, held: &FloorBands<Holding>) {
        for ((supply, capital), held) in self
            .supply_stored
            .iter_mut()
            .zip(self.capital_stored.iter_mut())
            .zip(held.iter())
        {
            supply.push(held.supply);
            capital.push(held.capital.to_cents());
        }
    }
}
