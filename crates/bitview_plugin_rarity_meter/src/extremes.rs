use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_types::{Bitcoin, Dollars, Height, StoredF32, Version};
use vecdb::{Database, Exit, ReadableVec, Rw, StorageMode};

use super::extreme::Extreme;
const VERSION: Version = Version::new(7);

#[derive(Traversable)]
pub struct Extremes<M: StorageMode = Rw> {
    /// Upper-tail extremeness of total all-chain supply in loss, in BTC, using
    /// all prior finite positive observations. Outputs require 210,000 accepted
    /// historical observations; thresholds exclude the current block, while
    /// the reported tail share includes it as one observation.
    pub coins_in_loss: Extreme<Bitcoin, M>,
    /// Upper-tail extremeness of trailing-24-hour all-chain realized profit, in
    /// USD, using all prior finite observations. Outputs require 210,000
    /// accepted historical observations; thresholds exclude the current block,
    /// while the reported tail share includes it as one observation.
    pub profit_taking: Extreme<Dollars, M>,
    /// Upper-tail extremeness of trailing-24-hour all-chain realized loss, in
    /// USD, using all prior finite observations. Outputs require 210,000
    /// accepted historical observations; thresholds exclude the current block,
    /// while the reported tail share includes it as one observation.
    pub capitulation: Extreme<Dollars, M>,
    /// Upper-tail extremeness of trailing-24-hour all-chain realized peak
    /// regret, in USD, using all prior finite observations. Outputs require
    /// 210,000 accepted historical observations; thresholds exclude the current
    /// block, while the reported tail share includes it as one observation.
    pub peak_regret: Extreme<Dollars, M>,
    /// Lower-tail extremeness of the trailing-24-hour all-chain sell-side risk
    /// ratio, expressed as a percentage, using the most recent 210,000 finite
    /// positive observations. Outputs require a full 210,000-observation
    /// history; thresholds exclude the current block, while the reported tail
    /// share includes it as one observation.
    pub seller_exhaustion: Extreme<StoredF32, M>,
}

pub fn forced_import(
    db: &Database,
    parent_version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
) -> Result<Extremes> {
    Extremes::forced_import(db, parent_version, mappings)
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    extremes: &mut Extremes,
    indexer: &Indexer,
    coins_in_loss: &impl ReadableVec<Height, Bitcoin>,
    realized_profit: &impl ReadableVec<Height, Dollars>,
    realized_loss: &impl ReadableVec<Height, Dollars>,
    peak_regret: &impl ReadableVec<Height, Dollars>,
    seller_exhaustion: &impl ReadableVec<Height, StoredF32>,
    exit: &Exit,
) -> Result<()> {
    extremes.compute(
        indexer,
        coins_in_loss,
        realized_profit,
        realized_loss,
        peak_regret,
        seller_exhaustion,
        exit,
    )
}

impl Extremes {
    fn forced_import(
        db: &Database,
        parent_version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        Ok(Self {
            coins_in_loss: Extreme::forced_import(
                db,
                "rarity_meter_coins_in_loss",
                version,
                mappings,
            )?,
            profit_taking: Extreme::forced_import(
                db,
                "rarity_meter_profit_taking",
                version,
                mappings,
            )?,
            capitulation: Extreme::forced_import(
                db,
                "rarity_meter_capitulation",
                version,
                mappings,
            )?,
            peak_regret: Extreme::forced_import(db, "rarity_meter_peak_regret", version, mappings)?,
            seller_exhaustion: Extreme::forced_import(
                db,
                "rarity_meter_seller_exhaustion",
                version,
                mappings,
            )?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn compute(
        &mut self,
        indexer: &Indexer,
        coins_in_loss: &impl ReadableVec<Height, Bitcoin>,
        realized_profit: &impl ReadableVec<Height, Dollars>,
        realized_loss: &impl ReadableVec<Height, Dollars>,
        peak_regret: &impl ReadableVec<Height, Dollars>,
        seller_exhaustion: &impl ReadableVec<Height, StoredF32>,
        exit: &Exit,
    ) -> Result<()> {
        self.coins_in_loss
            .compute_coins_in_loss(indexer, coins_in_loss, exit)?;
        self.profit_taking
            .compute_realized(indexer, realized_profit, exit)?;
        self.capitulation
            .compute_realized(indexer, realized_loss, exit)?;
        self.peak_regret
            .compute_realized(indexer, peak_regret, exit)?;
        self.seller_exhaustion
            .compute_seller_exhaustion(indexer, seller_exhaustion, exit)?;
        Ok(())
    }
}
