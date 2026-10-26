use bitview_compute::{prepare_computed, write_chunks, write_locked};
use bitview_primitives::{
    Lengths, PartsPerMillion32, PriceRatio, RARITY_PERCENTILES, RARITY_PERCENTILES_LEN,
};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, IndexSources, LazyRatioPerBlock, import_cached, shapes::RarityPercentiles,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, WritableVec,
};

use super::{Band, BlockDecayPercentiles, START_HEIGHT, component_price::ComponentPrice};

#[derive(Traversable)]
pub struct Component<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Historical valuation bands for judging how unusually low or high spot
    /// price is relative to this reference price. At each block, the
    /// spot-to-reference ratio is ranked against observations since height
    /// 210,000 with a 210,000-block backward half-life. Low percentiles represent
    /// rare low valuations and high percentiles rare high valuations. Ratios
    /// include the represented block, are rounded to 0.001, clamped from 0
    /// through 43, and exclude undefined values. A percentile below the median
    /// reads the lower edge of its 0.001 step and one above it the upper edge,
    /// so spot crosses a band only from a more extreme step, never by its own
    /// observation. Each price band equals the reference price multiplied by its
    /// historical ratio percentile.
    #[traversable(lead = "Rarity bands: prices as rare, against that reference price, as each percentile of its history, the low ones cheap, the high ones dear.")]
    pub bands: RarityPercentiles<Band>,

    /// The stored ratio percentiles behind `bands`, in parts per million.
    #[traversable(hidden)]
    ratios: RarityPercentiles<CachedSeries<Height, PartsPerMillion32, M>>,

    price: M::WriteOnly<ComponentPrice>,
    block_decay_pct: M::WriteOnly<BlockDecayPercentiles>,
}

const VERSION: Version = Version::new(15);

impl Component {
    /// `price` is the reference price's id: the bands are `<price>_rarity_<percentile>` and their
    /// ratios, percentiles of the price's spot ratio, `<price>_ratio_rarity_<percentile>`.
    pub(crate) fn import(
        db: &Database,
        price: &str,
        version: Version,
        mappings: &IndexSources,
        price_source: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + VERSION;
        let component_price = ComponentPrice::new(price_source);
        let ratios = RarityPercentiles::try_from_fn(|id| {
            import_cached(db, &format!("{price}_ratio_rarity_{}_ppm", id.suffix()), version)
        })?;
        let bands = RarityPercentiles::from_fn(|id| {
            let suffix = id.suffix();
            let ratio = LazyRatioPerBlock::from_height_source(
                &format!("{price}_ratio_rarity_{suffix}"),
                version,
                ratios.get(id),
                mappings,
            );
            let band = component_price.price_for_ratio(
                &format!("{price}_rarity_{suffix}"),
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
        ratio_source: &impl ReadableVec<Height, PriceRatio>,
        exit: &Exit,
    ) -> Result<()> {
        let source_end = ratio_source.len();
        let start = prepare_computed(
            self.ratios.iter_any_stored_mut().collect::<Vec<_>>(),
            ratio_source.version(),
            usize::from(starting_lengths.height).min(source_end),
            exit,
        )?;
        let Self {
            ratios,
            block_decay_pct,
            ..
        } = self;
        let expected_len = start.saturating_sub(START_HEIGHT);
        if block_decay_pct.len() != expected_len {
            block_decay_pct.reset();
            if start > START_HEIGHT {
                let historical = ratio_source.collect_range_at(START_HEIGHT, start);
                block_decay_pct.add_bulk(START_HEIGHT, &historical);
            }
        }
        let mut out = [0.0; RARITY_PERCENTILES_LEN];
        for chunk in write_chunks(start..source_end) {
            // Before calibration starts no ratio is observed: every percentile is undefined.
            let observed = chunk.start.max(START_HEIGHT).min(chunk.end);
            for _ in chunk.start..observed {
                for target in ratios.iter_mut() {
                    target.push(PartsPerMillion32::NAN);
                }
            }
            let mut height = observed;
            ratio_source.for_each_range_at(observed, chunk.end, |ratio| {
                block_decay_pct.add(height, f64::from(ratio));
                height += 1;
                block_decay_pct.quantiles(&RARITY_PERCENTILES, &mut out);
                for (target, value) in ratios.iter_mut().zip(out) {
                    target.push(PartsPerMillion32::from(value));
                }
            });
            write_locked(ratios.iter_any_stored_mut(), exit)?;
        }

        Ok(())
    }

    pub fn needs_compute(
        &self,
        starting_height: Height,
        ratio_source: &impl ReadableVec<Height, PriceRatio>,
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
        .expect("component boundary prices")
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
