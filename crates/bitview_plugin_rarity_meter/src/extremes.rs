use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Ratio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Dollars, Height, Version};
use vecdb::{Database, ReadableVec, Rw, StorageMode};

use super::extreme::{COINS_IN_LOSS, Extreme, REALIZED, SELL_SIDE_RISK};

const VERSION: Version = Version::new(7);

#[derive(Traversable)]
pub struct Extremes<M: StorageMode = Rw> {
    /// How unusually large all-chain supply in loss is: BTC whose creation price
    /// exceeds spot, in the upper tail of all prior finite positive
    /// observations, requiring 210,000 of them.
    pub supply_in_loss: Extreme<Bitcoin, M>,
    /// How unusually large trailing-24-hour all-chain realized profit is, in
    /// USD, in the upper tail of all prior finite observations, requiring
    /// 210,000 of them.
    pub realized_profit_24h: Extreme<Dollars, M>,
    /// How unusually large trailing-24-hour all-chain realized loss is, in USD,
    /// in the upper tail of all prior finite observations, requiring 210,000 of
    /// them.
    pub realized_loss_24h: Extreme<Dollars, M>,
    /// How unusually large trailing-24-hour all-chain realized peak regret is, in
    /// USD, in the upper tail of all prior finite observations, requiring
    /// 210,000 of them. Peak regret is the value sellers forgo relative to each
    /// spent output's highest spot price from its creation block through its
    /// spending block.
    pub realized_peak_regret_24h: Extreme<Dollars, M>,
    /// How unusually low the trailing-24-hour all-chain sell-side risk ratio
    /// (gross realized profit and loss divided by realized capitalization) is,
    /// in the lower tail of the most recent 210,000 finite positive
    /// observations, requiring a full window. Unusually low values indicate
    /// little realized profit or loss relative to invested value.
    pub sell_side_risk_ratio_24h: Extreme<Ratio, M>,
}

impl Extremes {
    pub(crate) fn import(
        db: &Database,
        parent_version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        Ok(Extremes {
            supply_in_loss: Extreme::import(db, "rarity_meter_supply_in_loss", version, mappings)?,
            realized_profit_24h: Extreme::import(
                db,
                "rarity_meter_realized_profit_24h",
                version,
                mappings,
            )?,
            realized_loss_24h: Extreme::import(
                db,
                "rarity_meter_realized_loss_24h",
                version,
                mappings,
            )?,
            realized_peak_regret_24h: Extreme::import(
                db,
                "rarity_meter_realized_peak_regret_24h",
                version,
                mappings,
            )?,
            sell_side_risk_ratio_24h: Extreme::import(
                db,
                "rarity_meter_sell_side_risk_ratio_24h",
                version + Version::ONE,
                mappings,
            )?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        coins_in_loss: &impl ReadableVec<Height, Bitcoin>,
        realized_profit: &impl ReadableVec<Height, Dollars>,
        realized_loss: &impl ReadableVec<Height, Dollars>,
        peak_regret: &impl ReadableVec<Height, Dollars>,
        sell_side_risk: &impl ReadableVec<Height, Ratio>,
        exit: &Exit,
    ) -> Result<()> {
        let from = indexer.compute_start().height;
        self.supply_in_loss
            .compute(from, coins_in_loss, COINS_IN_LOSS, exit)?;
        self.realized_profit_24h
            .compute(from, realized_profit, REALIZED, exit)?;
        self.realized_loss_24h
            .compute(from, realized_loss, REALIZED, exit)?;
        self.realized_peak_regret_24h
            .compute(from, peak_regret, REALIZED, exit)?;
        self.sell_side_risk_ratio_24h
            .compute(from, sell_side_risk, SELL_SIDE_RISK, exit)
    }
}
