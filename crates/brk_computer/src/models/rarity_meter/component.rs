use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{
    Cents, Height, PartsPerMillion32, RARITY_PERCENTILES, RARITY_PERCENTILES_LEN,
    RarityPercentileId, StoredF32, Version,
};
use vecdb::{
    AnyVec, ColumnId, ColumnarVec, Database, EagerVec, Exit, ImportableVec, PcoVec, ReadOnlyClone,
    ReadableCloneableVec, ReadableVec, Rw, StorageMode, WritableVec,
};

use super::{
    Band, BlockDecayPercentiles, COMPUTE_BATCH_SIZE, START_HEIGHT,
    cached_component_price::CachedComponentPrice, percentiles::RarityPercentiles,
};
use crate::{indexes, internal::LazyColumnRatioPerBlock};

#[derive(Traversable)]
pub struct Component<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Block-decay-weighted historical percentile bands of spot price divided
    /// by the component price named by the series. Observations begin at height
    /// 210,000, include the current block, and receive twice the weight every
    /// 210,000 blocks, equivalent to a 210,000-block backward half-life. Ratios
    /// are rounded to 0.001, clamped from 0 through 43, and NaNs are excluded.
    /// Percentiles are 0.1, 0.5, 1, 2, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90,
    /// 95, 98, 99, 99.5, and 99.9 percent. Each price band is the component price
    /// multiplied by its percentile ratio.
    pub bands: RarityPercentiles<Band>,

    /// Block-decay-weighted historical percentiles of spot price divided by the
    /// component price named by the series, stored in parts per million.
    /// Observations begin at height 210,000, include the current block, and
    /// receive twice the weight every 210,000 blocks, equivalent to a
    /// 210,000-block backward half-life. Ratios are rounded to 0.001, clamped
    /// from 0 through 43, and NaNs are excluded. Column order is 0.1, 0.5, 1, 2,
    /// 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 98, 99, 99.5, and 99.9
    /// percent.
    pub ratios:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, PartsPerMillion32>, RarityPercentileId>>>,

    #[traversable(skip)]
    block_decay_pct: BlockDecayPercentiles,

    #[traversable(skip)]
    cached_price: CachedComponentPrice,
}

const VERSION: Version = Version::new(11);

impl Component {
    pub(super) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
        price_source: &(impl ReadableCloneableVec<Height, Cents> + 'static),
    ) -> Result<Self> {
        let version = version + VERSION;
        let cached_price = CachedComponentPrice::new(name, version, price_source);
        let ratios = EagerVec::<
            ColumnarVec<PcoVec<Height, PartsPerMillion32>, RarityPercentileId>,
        >::forced_import(db, &format!("{name}_ratios_ppm"), version)?;
        let source = ratios.read_only_clone();
        let bands = RarityPercentiles::from_fn(|id| {
            let suffix = id.suffix();
            let ratio = LazyColumnRatioPerBlock::new(
                &format!("{name}_ratio_{suffix}"),
                version,
                &source,
                id,
                indexes,
            );
            let price = cached_price.price_for_ratio(
                &format!("{name}_{suffix}"),
                version,
                &ratio.ppm.height,
                indexes,
            );
            Band { ratio, price }
        });

        Ok(Self {
            bands,
            ratios,
            block_decay_pct: BlockDecayPercentiles::default(),
            cached_price,
        })
    }

    pub(super) fn compute(
        &mut self,
        starting_lengths: &Lengths,
        ratio_source: &impl ReadableVec<Height, StoredF32>,
        exit: &Exit,
    ) -> Result<()> {
        self.cached_price
            .clear_if_recomputed_from(starting_lengths.height);

        let block_decay_pct = &mut self.block_decay_pct;
        self.ratios.compute_batched_to(
            starting_lengths.height,
            ratio_source.len(),
            ratio_source.version(),
            COMPUTE_BATCH_SIZE,
            |ratios, range| {
                let expected_len = range.start.saturating_sub(START_HEIGHT);
                if block_decay_pct.len() != expected_len {
                    block_decay_pct.reset();
                    if range.start > START_HEIGHT {
                        let historical = ratio_source.collect_range_at(START_HEIGHT, range.start);
                        block_decay_pct.add_bulk(START_HEIGHT, &historical);
                    }
                }

                let new_ratios = ratio_source.collect_range_at(range.start, range.end);
                let mut out = [0.0; RARITY_PERCENTILES_LEN];
                for (offset, &ratio) in new_ratios.iter().enumerate() {
                    let height = range.start + offset;
                    if height >= START_HEIGHT {
                        block_decay_pct.add(height, *ratio);
                    }
                    block_decay_pct.quantiles(&RARITY_PERCENTILES, &mut out);
                    ratios.push(RarityPercentileId::from_fn(|id| {
                        PartsPerMillion32::from(out[id.index()])
                    }));
                }

                Ok(())
            },
            exit,
        )?;

        Ok(())
    }

    pub(super) fn boundary_version(&self) -> Version {
        self.bands
            .boundary_refs()
            .into_iter()
            .map(|band| band.price.cents.height.version())
            .sum()
    }

    pub(super) fn boundary_len(&self) -> usize {
        self.bands
            .boundary_refs()
            .into_iter()
            .map(|band| band.price.cents.height.len())
            .min()
            .unwrap_or_default()
    }

    pub(super) fn collect_boundary_prices(&self, start: usize, end: usize) -> [Vec<Cents>; 10] {
        self.bands
            .boundary_refs()
            .map(|band| band.price.cents.height.collect_range_at(start, end))
    }
}
