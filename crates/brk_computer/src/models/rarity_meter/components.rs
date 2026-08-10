use brk_error::Result;
use brk_indexer::{Indexer, Lengths};
use brk_traversable::Traversable;
use brk_types::{
    Cents, Height, PartsPerMillion32, RARITY_PERCENTILES, RARITY_PERCENTILES_LEN,
    RarityPercentileId, StoredF32, Version,
};
use vecdb::{
    AnyVec, ColumnId, ColumnarVec, Database, EagerVec, Exit, ImportableVec, PcoVec, ReadOnlyClone,
    ReadableVec, Rw, StorageMode, WritableVec,
};

use crate::{
    distribution,
    frameworks::{coinflow, cointime},
    indexes,
    internal::{LazyColumnRatioPerBlock, LazyPerBlock, Price},
};

use super::{
    COMPUTE_BATCH_SIZE,
    cached_component_price::CachedComponentPrice,
    percentiles::{BlockDecayPercentiles, RarityPercentiles, START_HEIGHT, suffix},
};

#[derive(Clone, Traversable)]
pub struct Band {
    #[traversable(flatten)]
    pub ratio: LazyColumnRatioPerBlock<PartsPerMillion32, RarityPercentileId>,
    pub price: Price<LazyPerBlock<Cents>>,
}

#[derive(Traversable)]
pub struct Component<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub bands: RarityPercentiles<Band>,

    pub ratios:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, PartsPerMillion32>, RarityPercentileId>>>,

    #[traversable(skip)]
    block_decay_pct: BlockDecayPercentiles,

    #[traversable(skip)]
    cached_price: CachedComponentPrice,
}

const VERSION: Version = Version::new(11);

impl Component {
    fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
        price_source: &(impl vecdb::ReadableCloneableVec<Height, Cents> + 'static),
    ) -> Result<Self> {
        let version = version + VERSION;
        let cached_price = CachedComponentPrice::new(name, version, price_source);
        let ratios = EagerVec::<
            ColumnarVec<PcoVec<Height, PartsPerMillion32>, RarityPercentileId>,
        >::forced_import(db, &format!("{name}_ratios_ppm"), version)?;
        let source = ratios.read_only_clone();
        let bands = RarityPercentiles::from_fn(|id| {
            let suffix = suffix(id);
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

    fn compute(
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

#[derive(Traversable)]
pub struct Components<M: StorageMode = Rw> {
    pub realized_price: Component<M>,
    pub capitalized_price: Component<M>,
    pub sth_realized_price: Component<M>,
    pub sth_capitalized_price: Component<M>,
    pub lth_realized_price: Component<M>,
    pub lth_capitalized_price: Component<M>,
    pub over_6m_realized_price: Component<M>,
    pub over_4m_realized_price: Component<M>,
    pub under_4m_realized_price: Component<M>,
    pub under_6m_realized_price: Component<M>,
    pub vaulted_price: Component<M>,
    pub active_price: Component<M>,
    pub true_market_mean_price: Component<M>,
    pub cointime_price: Component<M>,
    pub coinflow_price: Component<M>,
}

impl Components {
    pub(super) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        distribution: &distribution::Vecs,
        cointime: &cointime::Vecs,
        coinflow: &coinflow::Vecs,
    ) -> Result<Self> {
        let utxos = &distribution.utxo_cohorts;
        let all = &utxos.all.metrics.realized;
        let sth = &utxos.sth.metrics.realized;
        let lth = &utxos.lth.metrics.realized;

        macro_rules! import {
            ($name:expr, $source:expr) => {
                Component::forced_import(db, $name, version, indexes, &$source.cents.height)?
            };
        }

        Ok(Self {
            realized_price: import!("realized_price", all.price),
            capitalized_price: import!("capitalized_price", all.capitalized.price),
            sth_realized_price: import!("sth_realized_price", sth.price),
            sth_capitalized_price: import!("sth_capitalized_price", sth.capitalized.price),
            lth_realized_price: import!("lth_realized_price", lth.price),
            lth_capitalized_price: import!("lth_capitalized_price", lth.capitalized.price),
            over_6m_realized_price: import!(
                "over_6m_realized_price",
                utxos.over_age._6m.metrics.realized.price
            ),
            over_4m_realized_price: import!(
                "over_4m_realized_price",
                utxos.over_age._4m.metrics.realized.price
            ),
            under_4m_realized_price: import!(
                "under_4m_realized_price",
                utxos.under_age._4m.metrics.realized.price
            ),
            under_6m_realized_price: import!(
                "under_6m_realized_price",
                utxos.under_age._6m.metrics.realized.price
            ),
            vaulted_price: import!("vaulted_price", cointime.prices.vaulted),
            active_price: import!("active_price", cointime.prices.active),
            true_market_mean_price: import!(
                "true_market_mean_price",
                cointime.prices.true_market_mean
            ),
            cointime_price: import!("cointime_price", cointime.prices.cointime),
            coinflow_price: import!("coinflow_price", coinflow.all.price),
        })
    }

    pub(super) fn compute(
        &mut self,
        indexer: &Indexer,
        distribution: &distribution::Vecs,
        cointime: &cointime::Vecs,
        coinflow: &coinflow::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let utxos = &distribution.utxo_cohorts;
        let all = &utxos.all.metrics.realized;
        let sth = &utxos.sth.metrics.realized;
        let lth = &utxos.lth.metrics.realized;

        macro_rules! compute {
            ($component:ident, $source:expr) => {
                self.$component
                    .compute(&starting_lengths, &$source.ratio.height, exit)?;
            };
        }

        compute!(realized_price, &all.price);
        compute!(capitalized_price, &all.capitalized.price);
        compute!(sth_realized_price, &sth.price);
        compute!(sth_capitalized_price, &sth.capitalized.price);
        compute!(lth_realized_price, &lth.price);
        compute!(lth_capitalized_price, &lth.capitalized.price);
        compute!(
            over_6m_realized_price,
            &utxos.over_age._6m.metrics.realized.price
        );
        compute!(
            over_4m_realized_price,
            &utxos.over_age._4m.metrics.realized.price
        );
        compute!(
            under_4m_realized_price,
            &utxos.under_age._4m.metrics.realized.price
        );
        compute!(
            under_6m_realized_price,
            &utxos.under_age._6m.metrics.realized.price
        );
        compute!(vaulted_price, &cointime.prices.vaulted);
        compute!(active_price, &cointime.prices.active);
        compute!(true_market_mean_price, &cointime.prices.true_market_mean);
        compute!(cointime_price, &cointime.prices.cointime);
        compute!(coinflow_price, &coinflow.all.price);

        Ok(())
    }
}
