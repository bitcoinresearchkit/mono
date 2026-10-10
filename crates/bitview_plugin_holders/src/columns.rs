use bitview_cohort::AgeAggregateId;
use bitview_primitives::{CoinDays, Count, PartsPerMillionSigned32};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, CumulativeState, import_cached};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Height, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec, Rw, StorageMode, WritableVec};

use crate::{data::Data, unrealized_data::UnrealizedData};

#[derive(Traversable)]
pub(crate) struct Columns<M: StorageMode = Rw> {
    pub supply: CachedSeries<Height, Sats, M>,
    pub count: CachedSeries<Height, Count, M>,
    pub spent_count: CachedSeries<Height, Count, M>,
    pub volume_sats: CachedSeries<Height, Sats, M>,
    pub volume_cents: CachedSeries<Height, Cents, M>,
    pub volume_profit_sats: CachedSeries<Height, Sats, M>,
    pub volume_profit_cents: CachedSeries<Height, Cents, M>,
    pub volume_loss_sats: CachedSeries<Height, Sats, M>,
    pub volume_loss_cents: CachedSeries<Height, Cents, M>,
    pub adjusted_volume: CachedSeries<Height, Cents, M>,
    pub adjusted_value_destroyed: CachedSeries<Height, Cents, M>,
    pub cdd: CachedSeries<Height, CoinDays, M>,
    pub cap: CachedSeries<Height, Cents, M>,
    pub profit: CachedSeries<Height, Cents, M>,
    pub loss: CachedSeries<Height, Cents, M>,
    pub net_pnl: CachedSeries<Height, CentsSigned, M>,
    pub value_destroyed: CachedSeries<Height, Cents, M>,
    pub unrealized_profit: CachedSeries<Height, Cents, M>,
    pub unrealized_loss: CachedSeries<Height, Cents, M>,
    pub supply_profit: CachedSeries<Height, Sats, M>,
    pub supply_loss: CachedSeries<Height, Sats, M>,
    pub price: CachedSeries<Height, Cents, M>,
    pub capitalized_price: CachedSeries<Height, Cents, M>,
    pub gross_pnl: CachedSeries<Height, Cents, M>,
    pub peak_regret: CachedSeries<Height, Cents, M>,
    pub unrealized_net_pnl: CachedSeries<Height, CentsSigned, M>,
    pub nupl: CachedSeries<Height, PartsPerMillionSigned32, M>,
    pub unrealized_gross_pnl: CachedSeries<Height, Cents, M>,
    pub invested_profit: CachedSeries<Height, Cents, M>,
    pub invested_loss: CachedSeries<Height, Cents, M>,
    pub pain: CachedSeries<Height, Cents, M>,
    pub greed: CachedSeries<Height, Cents, M>,
    pub sentiment: CachedSeries<Height, CentsSigned, M>,
    peak: M::WriteOnly<CumulativeState<Cents>>,
}
impl Columns {
    pub fn import(db: &Database, id: AgeAggregateId, version: Version) -> Result<Self> {
        Ok(Self {
            supply: import_cached(db, &id.metric_name("supply_sats"), version)?,
            count: import_cached(db, &id.metric_name("utxo_count"), version)?,
            spent_count: import_cached(
                db,
                &id.metric_name("spent_utxo_count_cumulative"),
                version,
            )?,
            volume_sats: import_cached(
                db,
                &id.metric_name("transfer_volume_cumulative_sats"),
                version,
            )?,
            volume_cents: import_cached(
                db,
                &id.metric_name("transfer_volume_cumulative_cents"),
                version,
            )?,
            volume_profit_sats: import_cached(
                db,
                &id.metric_name("transfer_volume_in_profit_cumulative_sats"),
                version,
            )?,
            volume_profit_cents: import_cached(
                db,
                &id.metric_name("transfer_volume_in_profit_cumulative_cents"),
                version,
            )?,
            volume_loss_sats: import_cached(
                db,
                &id.metric_name("transfer_volume_in_loss_cumulative_sats"),
                version,
            )?,
            volume_loss_cents: import_cached(
                db,
                &id.metric_name("transfer_volume_in_loss_cumulative_cents"),
                version,
            )?,
            adjusted_volume: import_cached(
                db,
                &id.metric_name("adjusted_value_created_cumulative_cents"),
                version,
            )?,
            adjusted_value_destroyed: import_cached(
                db,
                &id.metric_name("adjusted_value_destroyed_cumulative_cents"),
                version,
            )?,
            cdd: import_cached(
                db,
                &id.metric_name("coindays_destroyed_cumulative"),
                version,
            )?,
            cap: import_cached(db, &id.metric_name("realized_cap_cents"), version)?,
            profit: import_cached(
                db,
                &id.metric_name("realized_profit_cumulative_cents"),
                version,
            )?,
            loss: import_cached(
                db,
                &id.metric_name("realized_loss_cumulative_cents"),
                version,
            )?,
            net_pnl: import_cached(
                db,
                &id.metric_name("net_realized_pnl_cumulative_cents"),
                version,
            )?,
            value_destroyed: import_cached(
                db,
                &id.metric_name("value_destroyed_cumulative_cents"),
                version,
            )?,
            unrealized_profit: import_cached(
                db,
                &id.metric_name("unrealized_profit_cents"),
                version,
            )?,
            unrealized_loss: import_cached(db, &id.metric_name("unrealized_loss_cents"), version)?,
            supply_profit: import_cached(db, &id.metric_name("supply_in_profit_sats"), version)?,
            supply_loss: import_cached(db, &id.metric_name("supply_in_loss_sats"), version)?,
            price: import_cached(db, &id.metric_name("realized_price_cents"), version)?,
            capitalized_price: import_cached(
                db,
                &id.metric_name("capitalized_price_cents"),
                version,
            )?,
            gross_pnl: import_cached(
                db,
                &id.metric_name("realized_gross_pnl_cumulative_cents"),
                version,
            )?,
            peak_regret: import_cached(
                db,
                &id.metric_name("realized_peak_regret_cumulative_cents"),
                version,
            )?,
            unrealized_net_pnl: import_cached(
                db,
                &id.metric_name("net_unrealized_pnl_cents"),
                version,
            )?,
            nupl: import_cached(db, &id.metric_name("nupl_ppm"), version + Version::ONE)?,
            unrealized_gross_pnl: import_cached(
                db,
                &id.metric_name("unrealized_gross_pnl_cents"),
                version,
            )?,
            invested_profit: import_cached(
                db,
                &id.metric_name("invested_capital_in_profit_cents"),
                version,
            )?,
            invested_loss: import_cached(
                db,
                &id.metric_name("invested_capital_in_loss_cents"),
                version,
            )?,
            pain: import_cached(db, &id.metric_name("pain_index_cents"), version)?,
            greed: import_cached(db, &id.metric_name("greed_index_cents"), version)?,
            sentiment: import_cached(db, &id.metric_name("net_sentiment_cents"), version)?,
            peak: Default::default(),
        })
    }
    pub fn push(&mut self, d: &Data, unrealized: &UnrealizedData) {
        let peak = self.peak.accumulate(
            self.peak_regret.len(),
            || self.peak_regret.collect_last(),
            |v| *v += d.peak_regret_raw.to_cents(),
        );
        self.supply.push(d.supply);
        self.count.push(d.count);
        self.spent_count.push(d.spent_count);
        self.volume_sats.push(d.volume_sats);
        self.volume_cents.push(d.volume_cents);
        self.volume_profit_sats.push(d.volume_profit_sats);
        self.volume_profit_cents.push(d.volume_profit_cents);
        self.volume_loss_sats.push(d.volume_loss_sats);
        self.volume_loss_cents.push(d.volume_loss_cents);
        self.adjusted_volume.push(d.adjusted_volume);
        self.adjusted_value_destroyed
            .push(d.adjusted_value_destroyed);
        self.cdd.push(d.cdd);
        self.cap.push(d.cap);
        self.profit.push(d.profit);
        self.loss.push(d.loss);
        self.net_pnl.push(d.net_pnl);
        self.value_destroyed.push(d.value_destroyed);
        self.unrealized_profit.push(d.unrealized_profit);
        self.unrealized_loss.push(d.unrealized_loss);
        self.supply_profit.push(d.supply_profit);
        self.supply_loss.push(d.supply_loss);
        self.price.push(Cents::new(
            d.cap_raw
                .as_u128()
                .checked_div(d.supply.as_u128())
                .unwrap_or_default() as u64,
        ));
        self.capitalized_price.push(Cents::new(
            d.capitalized_cap_raw
                .inner()
                .checked_div(d.cap_raw.as_u128())
                .unwrap_or_default() as u64,
        ));
        self.gross_pnl.push(d.profit + d.loss);
        self.peak_regret.push(peak);
        self.unrealized_net_pnl.push(unrealized.net_pnl);
        self.nupl.push(unrealized.nupl);
        self.unrealized_gross_pnl
            .push(d.unrealized_profit + d.unrealized_loss);
        self.invested_profit
            .push(unrealized.invested_capital_in_profit);
        self.invested_loss.push(unrealized.invested_capital_in_loss);
        self.pain.push(unrealized.pain_index);
        self.greed.push(unrealized.greed_index);
        self.sentiment.push(unrealized.net_sentiment);
    }
    pub fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.peak = Default::default();
        vec![
            &mut self.supply,
            &mut self.count,
            &mut self.spent_count,
            &mut self.volume_sats,
            &mut self.volume_cents,
            &mut self.volume_profit_sats,
            &mut self.volume_profit_cents,
            &mut self.volume_loss_sats,
            &mut self.volume_loss_cents,
            &mut self.adjusted_volume,
            &mut self.adjusted_value_destroyed,
            &mut self.cdd,
            &mut self.cap,
            &mut self.profit,
            &mut self.loss,
            &mut self.net_pnl,
            &mut self.value_destroyed,
            &mut self.unrealized_profit,
            &mut self.unrealized_loss,
            &mut self.supply_profit,
            &mut self.supply_loss,
            &mut self.price,
            &mut self.capitalized_price,
            &mut self.gross_pnl,
            &mut self.peak_regret,
            &mut self.unrealized_net_pnl,
            &mut self.nupl,
            &mut self.unrealized_gross_pnl,
            &mut self.invested_profit,
            &mut self.invested_loss,
            &mut self.pain,
            &mut self.greed,
            &mut self.sentiment,
        ]
    }
}
