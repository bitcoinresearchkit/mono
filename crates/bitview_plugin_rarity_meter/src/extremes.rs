use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Ratio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Dollars, Version};
use vecdb::{Database, Rw, StorageMode};

use super::extreme::{Extreme, REALIZED, SELL_SIDE_RISK, SUPPLY_IN_LOSS};

const VERSION: Version = Version::new(9);

#[derive(Traversable)]
pub struct Extremes<M: StorageMode = Rw> {
    /// All-chain supply in loss (BTC whose creation price exceeds spot), against all its prior
    /// positive observations, from 210,000 of them.
    pub supply_in_loss: Extreme<Bitcoin, M>,
    /// Trailing-24-hour all-chain realized profit, in USD, against all its prior finite
    /// observations, from 210,000 of them.
    pub realized_profit_sum_24h: Extreme<Dollars, M>,
    /// Trailing-24-hour all-chain realized loss, in USD, against all its prior finite
    /// observations, from 210,000 of them.
    pub realized_loss_sum_24h: Extreme<Dollars, M>,
    /// Trailing-24-hour all-chain realized peak regret, in USD, against all its prior finite
    /// observations, from 210,000 of them. Peak regret is the value sellers forgo relative to
    /// each spent output's highest spot price from its creation block through its spending block.
    pub realized_peak_regret_sum_24h: Extreme<Dollars, M>,
    /// The trailing-24-hour all-chain sell-side risk ratio (gross realized profit and loss
    /// divided by realized capitalization), against its most recent 210,000 positive
    /// observations. Low values mean little realized profit or loss relative to invested value.
    pub sell_side_risk_ratio_24h: Extreme<Ratio, M>,
}

impl Extremes {
    pub(crate) fn import(
        db: &Database,
        parent_version: Version,
        mappings: &MappingsVecs,
        holders: &HoldersVecs,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        let all = &holders.cohorts.all;
        let realized = &all.realized;
        Ok(Extremes {
            supply_in_loss: Extreme::import(
                db,
                &all.supply.in_loss.value.btc.height,
                version,
                mappings,
            )?,
            realized_profit_sum_24h: Extreme::import(
                db,
                &realized.profit.sum._24h.usd.height,
                version,
                mappings,
            )?,
            realized_loss_sum_24h: Extreme::import(
                db,
                &realized.loss.sum._24h.usd.height,
                version,
                mappings,
            )?,
            realized_peak_regret_sum_24h: Extreme::import(
                db,
                &realized.peak_regret.sum._24h.usd.height,
                version,
                mappings,
            )?,
            sell_side_risk_ratio_24h: Extreme::import(
                db,
                &realized.sell_side_risk_ratio._24h.ratio.height,
                version + Version::ONE,
                mappings,
            )?,
        })
    }

    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let from = indexer.compute_start().height;
        self.supply_in_loss.compute(from, SUPPLY_IN_LOSS, exit)?;
        self.realized_profit_sum_24h.compute(from, REALIZED, exit)?;
        self.realized_loss_sum_24h.compute(from, REALIZED, exit)?;
        self.realized_peak_regret_sum_24h
            .compute(from, REALIZED, exit)?;
        self.sell_side_risk_ratio_24h
            .compute(from, SELL_SIDE_RISK, exit)
    }
}
