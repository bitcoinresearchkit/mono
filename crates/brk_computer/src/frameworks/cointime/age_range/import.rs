use brk_cohort::{AgeRangeId, CohortContext};
use brk_error::Result;
use brk_types::{Cents, Height, StoredF64, Version};
use vecdb::{CachedBoxedVec, Database, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec};

use super::{ActivitySeries, SupplyVecs, Vecs};
use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlock, ColumnarPerBlockCumulativeRolling,
        LazyColumnPerBlock, LazyColumnPerBlockCumulativeRolling, LazyColumnSpotValuePerBlock,
        LazyPerBlock, OddsF64, OneMinusF64, Windows,
    },
};

const VERSION: Version = Version::new(3);

impl ActivitySeries {
    fn new(
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredF64>, AgeRangeId>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let wakefulness = AgeRangeId::series(CohortContext::Utxo, |column, name| {
            LazyColumnPerBlock::new(
                &format!("{name}_wakefulness"),
                version,
                source,
                column,
                indexes,
            )
        });
        let dormancy = AgeRangeId::series(CohortContext::Utxo, |column, name| {
            let wakefulness = column.select(&wakefulness);
            LazyPerBlock::from_resolutions::<OneMinusF64>(
                &format!("{name}_dormancy"),
                version,
                wakefulness.height.read_only_boxed_clone(),
                &wakefulness.resolutions,
            )
        });
        let wakefulness_to_dormancy = AgeRangeId::series(CohortContext::Utxo, |column, name| {
            let wakefulness = column.select(&wakefulness);
            LazyPerBlock::from_resolutions::<OddsF64>(
                &format!("{name}_wakefulness_to_dormancy"),
                version,
                wakefulness.height.read_only_boxed_clone(),
                &wakefulness.resolutions,
            )
        });

        Self {
            wakefulness,
            dormancy,
            wakefulness_to_dormancy,
        }
    }
}

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        parent_version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        let coindays_consumed = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &CohortContext::Utxo.prefixed("age_range_coindays_consumed_cumulative"),
            version,
            |source| {
                AgeRangeId::series(CohortContext::Utxo, |column, name| {
                    LazyColumnPerBlockCumulativeRolling::new(
                        &format!("{name}_coindays_consumed"),
                        version,
                        source,
                        column,
                        indexes,
                        cached_starts,
                    )
                })
            },
        )?;
        let coindays_stored = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            &CohortContext::Utxo.prefixed("age_range_coindays_stored_cumulative"),
            version,
            |source| {
                AgeRangeId::series(CohortContext::Utxo, |column, name| {
                    LazyColumnPerBlockCumulativeRolling::new(
                        &format!("{name}_coindays_stored"),
                        version,
                        source,
                        column,
                        indexes,
                        cached_starts,
                    )
                })
            },
        )?;
        let activity = ColumnarPerBlock::forced_import(
            db,
            &CohortContext::Utxo.prefixed("age_range_wakefulness"),
            version,
            |source| ActivitySeries::new(version, source, indexes),
        )?;
        let import_supply = |side: &str| {
            ColumnarPerBlock::forced_import(
                db,
                &CohortContext::Utxo.prefixed(&format!("age_range_{side}_supply_sats")),
                version,
                |source| {
                    AgeRangeId::series(CohortContext::Utxo, |column, name| {
                        LazyColumnSpotValuePerBlock::new(
                            &format!("{name}_{side}_supply"),
                            version,
                            source,
                            column,
                            indexes,
                            spot_price,
                        )
                    })
                },
            )
        };
        let supply = SupplyVecs {
            awake: import_supply("awake")?,
            dormant: import_supply("dormant")?,
        };

        Ok(Self {
            coindays_consumed,
            coindays_stored,
            activity,
            supply,
        })
    }
}
