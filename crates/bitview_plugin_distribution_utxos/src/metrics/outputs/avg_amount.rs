use bitview_cohort::SpendableType;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazySpotValuePerBlock, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode, WritableVec};

/// Mean unspent output value, calculated from the same block's supply and counts.
#[derive(Traversable)]
pub struct AvgAmount<M: StorageMode = Rw> {
    pub all: LazySpotValuePerBlock,
    pub by_type: SpendableType<LazySpotValuePerBlock>,
    #[traversable(hidden)]
    all_source: CachedSeries<Height, Sats, M>,
    #[traversable(hidden)]
    type_sources: SpendableType<CachedSeries<Height, Sats, M>>,
}

impl AvgAmount {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &Mappings,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let all_source = import_cached(db, "avg_utxo_amount_sats", version)?;
        let type_sources = SpendableType::try_new(|id| {
            import_cached(db, &format!("{}_avg_utxo_amount_sats", id.name()), version)
        })?;
        let all = LazySpotValuePerBlock::from_sats_source(
            "avg_utxo_amount",
            version,
            &all_source,
            mappings,
            spot,
        );
        let by_type = type_sources.map_with_id(|id, source| {
            LazySpotValuePerBlock::from_sats_source(
                &format!("{}_avg_utxo_amount", id.name()),
                version,
                source,
                mappings,
                spot,
            )
        });
        Ok(Self {
            all,
            by_type,
            all_source,
            type_sources,
        })
    }

    pub fn push(
        &mut self,
        supplies: &SpendableType<Sats>,
        counts: &SpendableType<(StoredU64, StoredU64)>,
    ) {
        let mut total = Sats::ZERO;
        let mut count = 0u64;
        for ((target, &sats), &(unspent, _)) in self
            .type_sources
            .iter_mut()
            .zip(supplies.iter())
            .zip(counts.iter())
        {
            target.push(sats / unspent);
            total += sats;
            count += u64::from(unspent);
        }
        self.all_source.push(total / StoredU64::from(count));
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.type_sources
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .chain([&mut self.all_source as &mut dyn AnyStoredVec])
    }
}
