use std::ops::AddAssign;

use brk_cohort::{TermId, UTXOAggregateId};
use brk_error::Result;
use brk_types::{Cents, Height, StoredF64, Version};
use vecdb::{
    CachedBoxedVec, ColumnId, Database, ImportableVec, PcoVec, PcoVecValue, ReadOnlyClone,
    ReadOnlyColumnarVec, ReadableBoxedVec, ReadableCloneableVec, ReadableColumnarVec,
};

use super::{AwakeVecs, CohortVecs, DormantVecs, Sources, Vecs};
use crate::{
    indexes,
    internal::{
        Identity, LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock,
        LazySpotValuePerBlock, PerBlock,
    },
};

impl Sources {
    fn forced_import(db: &Database, version: Version) -> Result<Self> {
        Ok(Self {
            awake_supply: ImportableVec::forced_import(
                db,
                "cointime_awake_supply_sats_by_term",
                version,
            )?,
            dormant_supply: ImportableVec::forced_import(
                db,
                "cointime_dormant_supply_sats_by_term",
                version,
            )?,
            awake_cap: ImportableVec::forced_import(
                db,
                "cointime_awake_cap_cents_by_term",
                version,
            )?,
            awake_price: ImportableVec::forced_import(
                db,
                "cointime_awake_price_cents_by_aggregate",
                version,
            )?,
            supply_in_loss_share: ImportableVec::forced_import(
                db,
                "cointime_awake_supply_in_loss_share_by_term",
                version,
            )?,
        })
    }

    fn additive_source<T>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, TermId>,
        name: &str,
        version: Version,
        aggregate: UTXOAggregateId,
    ) -> ReadableBoxedVec<Height, T>
    where
        T: PcoVecValue + AddAssign,
    {
        match aggregate.term() {
            Some(term) => source.column(name, version, term).read_only_boxed_clone(),
            None => source
                .sum_columns(name, version, TermId::ALL.iter().copied())
                .read_only_boxed_clone(),
        }
    }
}

impl CohortVecs {
    fn new(
        aggregate: UTXOAggregateId,
        version: Version,
        sources: &Sources,
        supply_in_loss_share: ReadableBoxedVec<Height, StoredF64>,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let name = aggregate.cohort_name().id;
        let prefix = if name.is_empty() {
            String::new()
        } else {
            format!("{name}_")
        };
        let awake_supply = Sources::additive_source(
            &sources.awake_supply.read_only_clone(),
            &format!("{prefix}awake_supply_sats"),
            version,
            aggregate,
        );
        let dormant_supply = Sources::additive_source(
            &sources.dormant_supply.read_only_clone(),
            &format!("{prefix}dormant_supply_sats"),
            version,
            aggregate,
        );
        let awake_cap = Sources::additive_source(
            &sources.awake_cap.read_only_clone(),
            &format!("{prefix}awake_cap_cents"),
            version,
            aggregate,
        );
        let awake_price = sources
            .awake_price
            .read_only_clone()
            .column(&format!("{prefix}awake_price_cents"), version, aggregate)
            .read_only_boxed_clone();

        Self {
            awake: AwakeVecs {
                supply: LazySpotValuePerBlock::from_boxed_sats_source(
                    &format!("{prefix}awake_supply"),
                    version,
                    awake_supply,
                    indexes,
                    spot_price,
                ),
                supply_in_loss_share: LazyPerBlock::from_uncached_boxed_height_source::<
                    Identity<StoredF64>,
                >(
                    &format!("{prefix}awake_supply_in_loss_share"),
                    version,
                    supply_in_loss_share,
                    indexes,
                ),
                cap: LazyFiatPerBlock::from_boxed_cents_source(
                    &format!("{prefix}awake_cap"),
                    version,
                    awake_cap,
                    indexes,
                ),
                price: LazyPriceWithRatioPerBlock::from_boxed_height_source(
                    &format!("{prefix}awake_price"),
                    version,
                    awake_price,
                    indexes,
                    spot_price,
                ),
            },
            dormant: DormantVecs {
                supply: LazySpotValuePerBlock::from_boxed_sats_source(
                    &format!("{prefix}dormant_supply"),
                    version,
                    dormant_supply,
                    indexes,
                    spot_price,
                ),
            },
        }
    }
}

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
        all_supply_in_loss_share: &PerBlock<StoredF64>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let sources = Sources::forced_import(db, version)?;
        let all_loss_share = all_supply_in_loss_share.height.read_only_boxed_clone();
        let term_loss_share = |aggregate: UTXOAggregateId| {
            let name = aggregate.cohort_name().id;
            debug_assert!(aggregate.term().is_some());
            Sources::additive_source(
                &sources.supply_in_loss_share.read_only_clone(),
                &format!("{name}_awake_supply_in_loss_share"),
                version,
                aggregate,
            )
        };
        let all = CohortVecs::new(
            UTXOAggregateId::All,
            version,
            &sources,
            all_loss_share,
            indexes,
            spot_price,
        );
        let sth = CohortVecs::new(
            UTXOAggregateId::Sth,
            version,
            &sources,
            term_loss_share(UTXOAggregateId::Sth),
            indexes,
            spot_price,
        );
        let lth = CohortVecs::new(
            UTXOAggregateId::Lth,
            version,
            &sources,
            term_loss_share(UTXOAggregateId::Lth),
            indexes,
            spot_price,
        );

        Ok(Self {
            all,
            sth,
            lth,
            sources,
        })
    }
}
