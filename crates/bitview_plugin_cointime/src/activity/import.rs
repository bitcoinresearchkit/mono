use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{CoinBlocks, PartsPerMillion64};
use bitview_vecs::{
    LazyPerBlock, LazyPerBlockCumulativeRolling, LazyRatioRollingWindows,
    LazyRollingSumsFromHeight, LazyWindowStartVec, PerBlock, PerBlockCumulative,
    PerBlockCumulativeRolling,
    ops::{BoundedOdds, BoundedToRatio, Quotient},
    shapes::Windows,
};
use brk_error::Result;
use brk_types::{Bitcoin, Height, Sats, Version};
use vecdb::{Database, LazyVec, ReadableBoxedVec, ReverseOperands};

use super::{CoinblocksCreated, DerivedVecs, Vecs};

impl CoinblocksCreated {
    fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        circulating_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        const NAME: &str = "coinblocks_created";
        let block = LazyVec::new(NAME, version, circulating_supply.clone(), |_, sats| {
            CoinBlocks::new(f64::from(Bitcoin::from(sats)))
        });
        let running = PerBlockCumulative::import(db, NAME, version, mappings)?;
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{NAME}_sum"),
            version,
            running.cumulative_source(),
            window_starts,
            mappings,
        );
        Ok(Self {
            block,
            running,
            sum,
        })
    }
}

impl DerivedVecs {
    fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let version = version + Version::ONE;
        let liveliness_source =
            PerBlock::import(db, "liveliness_bounded_source", version, mappings)?;
        let liveliness = LazyPerBlock::from_resolutions::<BoundedToRatio>(
            "liveliness",
            version,
            &liveliness_source,
        );
        let vaultedness = LazyPerBlock::from_resolutions::<BoundedToRatio<true>>(
            "vaultedness",
            version,
            &liveliness_source,
        );
        let liveliness_to_vaultedness = LazyPerBlock::from_resolutions::<BoundedOdds>(
            "liveliness_to_vaultedness",
            version + Version::ONE,
            &liveliness_source,
        );
        Ok(Self {
            liveliness,
            vaultedness,
            liveliness_to_vaultedness,
            liveliness_source,
        })
    }
}

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        age: &AgeVecs,
        circulating_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let coinblocks_created =
            CoinblocksCreated::import(db, version, mappings, window_starts, circulating_supply)?;
        // Destroyed over created, reading the created blocks first.
        let concurrent_liveliness = LazyRatioRollingWindows::from_cumulative_ratio::<
            CoinBlocks,
            CoinBlocks,
            ReverseOperands<Quotient<PartsPerMillion64>>,
        >(
            "concurrent_liveliness",
            version,
            coinblocks_created.cumulative_source(),
            age.coinblocks_destroyed.cumulative_source(),
            window_starts,
            mappings,
        );
        let coinblocks_destroyed = LazyPerBlockCumulativeRolling::from_cumulative_source(
            "coinblocks_destroyed",
            version,
            age.coinblocks_destroyed.cumulative_source(),
            window_starts,
            mappings,
        );
        Ok(Vecs {
            coinblocks_created,
            coinblocks_destroyed,
            coinblocks_stored: PerBlockCumulativeRolling::import(
                db,
                "coinblocks_stored",
                version,
                mappings,
                window_starts,
            )?,
            derived: DerivedVecs::import(db, version, mappings)?,
            concurrent_liveliness,
        })
    }
}
