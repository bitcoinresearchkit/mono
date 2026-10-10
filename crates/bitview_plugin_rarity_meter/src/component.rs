use bitview_collections::RarityPercentiles;
use bitview_primitives::Lengths;
use bitview_primitives::{PartsPerMillion32, RARITY_PERCENTILES, RARITY_PERCENTILES_LEN, Ratio};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, IndexSources, LazyRatioPerBlock, import_cached};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, WritableVec,
};

use super::{
    Band, BlockDecayPercentiles, COMPUTE_BATCH_SIZE, START_HEIGHT, component_price::ComponentPrice,
};

#[derive(Traversable)]
pub struct Component<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Historical valuation bands for judging how unusually low or high spot
    /// price is relative to this reference price. At each block, the
    /// spot-to-reference ratio is ranked against observations since height
    /// 210,000 with a 210,000-block backward half-life. Low percentiles represent
    /// rare low valuations and high percentiles rare high valuations. Ratios
    /// include the represented block, are rounded to 0.001, clamped from 0
    /// through 43, and exclude undefined values. Each price band equals the reference price
    /// multiplied by its historical ratio percentile.
    pub bands: RarityPercentiles<Band>,

    /// Raw historical spot-to-reference ratio percentiles behind this Rarity
    /// Meter component, stored in parts per million. Low percentiles represent
    /// rare low valuations and high percentiles rare high valuations.
    /// Observations begin at height 210,000, include the represented block, and
    /// use a 210,000-block backward half-life. Ratios are rounded to 0.001,
    /// clamped from 0 through 43, and exclude undefined values. Percentiles are 0.1, 0.5, 1,
    /// 2, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 98, 99, 99.5, and 99.9
    /// percent.
    #[traversable(hidden)]
    pub ratios: RarityPercentiles<CachedSeries<Height, PartsPerMillion32, M>>,

    price: M::WriteOnly<ComponentPrice>,
    block_decay_pct: M::WriteOnly<BlockDecayPercentiles>,
}

const VERSION: Version = Version::new(12);

impl Component {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &IndexSources,
        price_source: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + VERSION;
        let component_price = ComponentPrice::new(name, version, price_source);
        let ratios = RarityPercentiles::try_from_fn(|id| {
            import_cached(db, &format!("{name}_ratio_{}_ppm", id.suffix()), version)
        })?;
        let bands = RarityPercentiles::from_fn(|id| {
            let suffix = id.suffix();
            let ratio = LazyRatioPerBlock::from_height_source(
                &format!("{name}_ratio_{suffix}"),
                version,
                ratios.get(id),
                mappings,
            );
            let band = component_price.price_for_ratio(
                &format!("{name}_band_{suffix}"),
                version,
                &ratio.fixed.height,
                mappings,
            );
            Band { ratio, band }
        });

        Ok(Component {
            bands,
            ratios,
            price: component_price,
            block_decay_pct: BlockDecayPercentiles::default(),
        })
    }

    pub(crate) fn compute(
        &mut self,
        starting_lengths: &Lengths,
        ratio_source: &impl ReadableVec<Height, Ratio>,
        exit: &Exit,
    ) -> Result<()> {
        let block_decay_pct = &mut self.block_decay_pct;
        {
            let _lock = exit.lock();
            for vec in self.ratios.iter_mut() {
                vec.validate_computed_version_or_reset(ratio_source.version())?;
            }
        }
        let start = self
            .ratios
            .iter()
            .map(|v| v.len())
            .min()
            .unwrap_or_default()
            .min(usize::from(starting_lengths.height))
            .min(ratio_source.len());
        {
            let _lock = exit.lock();
            for vec in self.ratios.iter_mut() {
                vec.truncate_if_needed_at(start)?;
                vec.write()?;
            }
        }
        let expected_len = start.saturating_sub(START_HEIGHT);
        if block_decay_pct.len() != expected_len {
            block_decay_pct.reset();
            if start > START_HEIGHT {
                let historical = ratio_source.collect_range_at(START_HEIGHT, start);
                block_decay_pct.add_bulk(START_HEIGHT, &historical);
            }
        }
        let mut chunk_start = start;
        while chunk_start < ratio_source.len() {
            let end = (chunk_start + COMPUTE_BATCH_SIZE).min(ratio_source.len());
            let new_ratios = ratio_source.collect_range_at(chunk_start, end);
            let mut out = [0.0; RARITY_PERCENTILES_LEN];
            for (offset, ratio) in new_ratios.iter().enumerate() {
                let height = chunk_start + offset;
                if height >= START_HEIGHT {
                    block_decay_pct.add(height, **ratio);
                }
                block_decay_pct.quantiles(&RARITY_PERCENTILES, &mut out);
                for (target, value) in self.ratios.iter_mut().zip(out) {
                    target.push(PartsPerMillion32::from(value));
                }
            }
            let _lock = exit.lock();
            for vec in self.ratios.iter_mut() {
                vec.write()?;
            }
            chunk_start = end;
        }

        Ok(())
    }

    pub fn needs_compute(
        &self,
        starting_height: Height,
        ratio_source: &impl ReadableVec<Height, Ratio>,
    ) -> bool {
        self.ratios.iter().any(|v| {
            v.len() != ratio_source.len()
                || v.version() != v.header().vec_version() + ratio_source.version()
                || v.len() > usize::from(starting_height)
        })
    }
}

pub fn boundary_version(component: &Component) -> Version {
    component
        .bands
        .boundary_refs()
        .into_iter()
        .map(|band| band.band.cents.height.version())
        .sum()
}

pub fn boundary_len(component: &Component) -> usize {
    component
        .bands
        .boundary_refs()
        .into_iter()
        .map(|band| band.band.cents.height.len())
        .min()
        .unwrap_or_default()
}

pub fn collect_boundary_prices(
    component: &Component,
    start: usize,
    end: usize,
) -> [Vec<Cents>; 10] {
    component
        .price
        .collect_boundary_prices(component.ratios.boundary_refs(), start, end)
}
