use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Ratio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Bitcoin, Dollars, Height, Version};
use vecdb::{Database, ReadableVec, Rw, StorageMode};

use super::extreme::Extreme;

const VERSION: Version = Version::new(7);

#[derive(Traversable)]
pub struct Extremes<M: StorageMode = Rw> {
    /// Measures how unusually large the total all-chain supply in loss is. The
    /// source is BTC whose creation price exceeds spot; its upper-tail share is
    /// the fraction of accepted history at least this high, so a smaller share
    /// means a rarer event. Uses all prior finite positive observations and
    /// requires 210,000 of them. Thresholds exclude the represented block; the
    /// reported tail share includes it as one observation.
    pub supply_in_loss: Extreme<Bitcoin, M>,
    /// Measures how unusually large trailing-24-hour all-chain realized profit
    /// is in USD. Its upper-tail share is the fraction of accepted history at
    /// least this high, so a smaller share means rarer profit taking. Uses all
    /// prior finite observations and requires 210,000 of them. Thresholds
    /// exclude the represented block; the reported tail share includes it as
    /// one observation.
    pub realized_profit_24h: Extreme<Dollars, M>,
    /// Measures how unusually large trailing-24-hour all-chain realized loss is
    /// in USD. Its upper-tail share is the fraction of accepted history at least
    /// this high, so a smaller share means rarer capitulation. Uses all prior
    /// finite observations and requires 210,000 of them. Thresholds exclude the
    /// represented block; the reported tail share includes it as one
    /// observation.
    pub realized_loss_24h: Extreme<Dollars, M>,
    /// Measures how unusually large trailing-24-hour all-chain realized peak
    /// regret is in USD. Peak regret is the value sellers forgo relative to each
    /// spent output's highest spot price from its creation block through its
    /// spending block. The upper-tail share is the fraction of accepted history
    /// at least this high, so a smaller share means a rarer event. Uses all prior
    /// finite observations and requires 210,000 of them. Thresholds exclude the
    /// represented block; the reported tail share includes it as one
    /// observation.
    pub realized_peak_regret_24h: Extreme<Dollars, M>,
    /// Measures how unusually low the trailing-24-hour all-chain sell-side risk
    /// ratio is. That ratio is gross realized profit and loss divided by
    /// realized capitalization; unusually low values indicate little realized
    /// profit or loss relative to invested value. The lower-tail share is the
    /// fraction of accepted history at or below the source value, so a smaller
    /// share means stronger seller exhaustion. Uses the most recent 210,000
    /// finite positive observations and requires a full window. Thresholds
    /// exclude the represented block; the reported tail share includes it as
    /// one observation.
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
        seller_exhaustion: &impl ReadableVec<Height, Ratio>,
        exit: &Exit,
    ) -> Result<()> {
        self.supply_in_loss
            .compute_coins_in_loss(indexer, coins_in_loss, exit)?;
        self.realized_profit_24h
            .compute_realized(indexer, realized_profit, exit)?;
        self.realized_loss_24h
            .compute_realized(indexer, realized_loss, exit)?;
        self.realized_peak_regret_24h
            .compute_realized(indexer, peak_regret, exit)?;
        self.sell_side_risk_ratio_24h.compute_seller_exhaustion(
            indexer,
            seller_exhaustion,
            exit,
        )?;
        Ok(())
    }
}
