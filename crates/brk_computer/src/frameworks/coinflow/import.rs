use brk_cohort::{AgeRangeId, CohortContext, TERM_NAMES};
use brk_error::Result;
use brk_types::{Cents, Height, StoredF64, Version};
use vecdb::{
    CachedBoxedVec, Database, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec, UnaryTransform,
};

use super::{
    AgeRangeVecs, AggregateVecs, HorizonVecs, Horizons, SpendingExposureSeries, Split, Vecs,
    mobility,
};
use crate::{
    indexes,
    internal::{
        ColumnarPerBlock, FiatPerBlock, LazyColumnPerBlock, LazyColumnSpotValuePerBlock,
        LazyPerBlock, PerBlock, PriceWithRatioPerBlock, SpotValuePerBlock,
    },
};

const VERSION: Version = Version::new(4);

struct ExposureToMobility;

impl UnaryTransform<StoredF64, StoredF64> for ExposureToMobility {
    #[inline(always)]
    fn apply(exposure: StoredF64) -> StoredF64 {
        StoredF64::from(mobility(*exposure))
    }
}

impl SpendingExposureSeries {
    fn new(
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredF64>, AgeRangeId>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let age_range = AgeRangeId::series(CohortContext::Utxo, |column, name| {
            LazyColumnPerBlock::new(
                &format!("{name}_spending_exposure"),
                version,
                source,
                column,
                indexes,
            )
        });
        let mobility = AgeRangeId::series(CohortContext::Utxo, |column, name| {
            let exposure = column.select(&age_range);
            LazyPerBlock::from_resolutions::<ExposureToMobility>(
                &format!("{name}_mobility"),
                version,
                exposure.height.read_only_boxed_clone(),
                &exposure.resolutions,
            )
        });

        Self {
            age_range,
            mobility,
        }
    }
}

impl AggregateVecs {
    fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let prefix = if name.is_empty() {
            String::new()
        } else {
            format!("{name}_")
        };

        Ok(Self {
            supply: Split::try_from_fn(|side| {
                SpotValuePerBlock::forced_import(
                    db,
                    &format!("{prefix}{side}_supply"),
                    version,
                    indexes,
                    spot_price,
                )
            })?,
            supply_in_loss_share: PerBlock::forced_import(
                db,
                &format!("{prefix}coinflow_supply_in_loss_share"),
                version,
                indexes,
            )?,
            horizon: Horizons::try_from_fn(|horizon, _| -> Result<_> {
                Ok(HorizonVecs {
                    supply_in_loss_share: PerBlock::forced_import(
                        db,
                        &format!("{prefix}coinflow_{horizon}_supply_in_loss_share"),
                        version,
                        indexes,
                    )?,
                })
            })?,
            cap: FiatPerBlock::forced_import(
                db,
                &format!("{prefix}coinflow_cap"),
                version,
                indexes,
            )?,
            price: PriceWithRatioPerBlock::forced_import(
                db,
                &format!("{prefix}coinflow_price"),
                version,
                indexes,
                spot_price,
            )?,
        })
    }
}

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        parent_version: Version,
        indexes: &indexes::Vecs,
        prices: &crate::price::Vecs,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        let aggregate_version = version + Version::ONE;
        let spot_price = prices.spot.cents.height.read_only_cached_boxed_clone();
        let spending_rate = ColumnarPerBlock::forced_import(
            db,
            &CohortContext::Utxo.prefixed("age_range_spending_rate"),
            version,
            |source| {
                AgeRangeId::series(CohortContext::Utxo, |column, name| {
                    LazyColumnPerBlock::new(
                        &format!("{name}_spending_rate"),
                        version,
                        source,
                        column,
                        indexes,
                    )
                })
            },
        )?;
        let spending_exposure = ColumnarPerBlock::forced_import(
            db,
            &CohortContext::Utxo.prefixed("age_range_spending_exposure"),
            version,
            |source| SpendingExposureSeries::new(version, source, indexes),
        )?;
        let supply = Split::try_from_fn(|side| {
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
                            &spot_price,
                        )
                    })
                },
            )
        })?;

        let this = Self {
            age_range: AgeRangeVecs {
                spending_rate,
                spending_exposure,
                supply,
            },
            all: AggregateVecs::forced_import(db, "", version, indexes, &spot_price)?,
            sth: AggregateVecs::forced_import(
                db,
                TERM_NAMES.short.id,
                aggregate_version,
                indexes,
                &spot_price,
            )?,
            lth: AggregateVecs::forced_import(
                db,
                TERM_NAMES.long.id,
                aggregate_version,
                indexes,
                &spot_price,
            )?,
        };

        Ok(this)
    }
}
