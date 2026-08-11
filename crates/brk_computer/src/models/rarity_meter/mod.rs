mod cached_component_price;
mod components;
mod extremes;
mod inner;
mod percentiles;

use brk_error::Result;
use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Exit, Rw, StorageMode};

use crate::{
    distribution,
    frameworks::{coinflow, cointime},
    indexes, price,
};

pub use components::{Component, Components};
pub use extremes::Extremes;
pub use inner::RarityMeterInner;

#[derive(Traversable)]
pub struct RarityMeter<M: StorageMode = Rw> {
    pub components: Components<M>,
    pub extremes: Extremes<M>,
    pub full: RarityMeterInner<M>,
    pub local: RarityMeterInner<M>,
    pub cycle: RarityMeterInner<M>,
}

const VERSION: Version = Version::new(7);
const COMPUTE_BATCH_SIZE: usize = 100_000;

impl RarityMeter {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        distribution: &distribution::Vecs,
        cointime: &cointime::Vecs,
        coinflow: &coinflow::Vecs,
    ) -> Result<Self> {
        let v = version + VERSION;
        Ok(Self {
            components: Components::forced_import(
                db,
                v,
                indexes,
                distribution,
                cointime,
                coinflow,
            )?,
            extremes: Extremes::forced_import(db, v, indexes)?,
            full: RarityMeterInner::forced_import(db, "rarity_meter", v, indexes)?,
            local: RarityMeterInner::forced_import(db, "local_rarity_meter", v, indexes)?,
            cycle: RarityMeterInner::forced_import(db, "cycle_rarity_meter", v, indexes)?,
        })
    }

    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        distribution: &distribution::Vecs,
        cointime: &cointime::Vecs,
        coinflow: &coinflow::Vecs,
        prices: &price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let spot = &prices.spot.cents.height;
        let metrics = &distribution.cohorts;
        let realized = &metrics.realized;

        self.components
            .compute(indexer, distribution, cointime, coinflow, exit)?;
        self.extremes.compute(
            indexer,
            &metrics.supply.in_loss.cohorts.all.btc.height,
            &realized.profit.cohorts.all.sum._24h.usd.height,
            &realized.loss.cohorts.all.sum._24h.usd.height,
            &realized.peak_regret.series.all.sum._24h.usd.height,
            &realized.sell_side_risk_ratio.all._24h.percent.height,
            exit,
        )?;

        // Full: all Rainbow components, 10 models
        self.full.compute(
            &[
                &self.components.under_4m_realized_price,
                &self.components.under_6m_realized_price,
                &self.components.over_4m_realized_price,
                &self.components.over_6m_realized_price,
                &self.components.sth_realized_price,
                &self.components.sth_capitalized_price,
                &self.components.lth_realized_price,
                &self.components.lth_capitalized_price,
                &self.components.realized_price,
                &self.components.capitalized_price,
            ],
            spot,
            indexer,
            exit,
        )?;

        // Local: young-coin and STH components, 4 models
        self.local.compute(
            &[
                &self.components.under_4m_realized_price,
                &self.components.under_6m_realized_price,
                &self.components.sth_realized_price,
                &self.components.sth_capitalized_price,
            ],
            spot,
            indexer,
            exit,
        )?;

        // Cycle: old-coin, all, and LTH components, 6 models
        self.cycle.compute(
            &[
                &self.components.over_4m_realized_price,
                &self.components.over_6m_realized_price,
                &self.components.realized_price,
                &self.components.capitalized_price,
                &self.components.lth_realized_price,
                &self.components.lth_capitalized_price,
            ],
            spot,
            indexer,
            exit,
        )?;

        Ok(())
    }
}
