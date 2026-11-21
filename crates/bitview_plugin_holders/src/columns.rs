use bitview_cohort::AgeAggregateId;
use bitview_primitives::{CoinDays, Count, PartsPerMillionSigned32};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, CumulativeState, IndexSources, import_cached, ops::Quotient};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Sats, Version};
use vecdb::{AnyVec, BinaryTransform, Database, ReadableVec, Rw, StorageMode, WritableVec};

use crate::accounting::AccountingTotals;

#[derive(Traversable)]
pub(crate) struct Columns<M: StorageMode = Rw> {
    pub supply: CachedSeries<Sats, M>,
    pub count: CachedSeries<Count, M>,
    pub spent_count: CachedSeries<Count, M>,
    pub volume_sats: CachedSeries<Sats, M>,
    pub volume_cents: CachedSeries<Cents, M>,
    pub volume_profit_sats: CachedSeries<Sats, M>,
    pub volume_profit_cents: CachedSeries<Cents, M>,
    pub volume_loss_cents: CachedSeries<Cents, M>,
    pub cdd: CachedSeries<CoinDays, M>,
    pub cap: CachedSeries<Cents, M>,
    pub profit: CachedSeries<Cents, M>,
    pub loss: CachedSeries<Cents, M>,
    pub value_destroyed: CachedSeries<Cents, M>,
    pub unrealized_profit: CachedSeries<Cents, M>,
    pub unrealized_loss: CachedSeries<Cents, M>,
    pub supply_profit: CachedSeries<Sats, M>,
    pub price: CachedSeries<Cents, M>,
    pub capitalized_price: CachedSeries<Cents, M>,
    pub peak_regret: CachedSeries<Cents, M>,
    pub nupl: CachedSeries<PartsPerMillionSigned32, M>,
    pub cap_profit: CachedSeries<Cents, M>,
    pub cap_loss: CachedSeries<Cents, M>,
    peak: M::WriteOnly<CumulativeState<Cents>>,
}
impl Columns {
    pub fn import(
        db: &Database,
        id: AgeAggregateId,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            supply: import_cached(db, &id.metric_name("supply_sats"), version, indexes)?,
            count: import_cached(db, &id.metric_name("utxo_count"), version, indexes)?,
            spent_count: import_cached(
                db,
                &id.metric_name("spent_utxo_count_cumulative"),
                version,
                indexes,
            )?,
            volume_sats: import_cached(
                db,
                &id.metric_name("transfer_volume_cumulative_sats"),
                version,
                indexes,
            )?,
            volume_cents: import_cached(
                db,
                &id.metric_name("transfer_volume_cumulative_cents"),
                version,
                indexes,
            )?,
            volume_profit_sats: import_cached(
                db,
                &id.metric_name("transfer_volume_in_profit_cumulative_sats"),
                version,
                indexes,
            )?,
            volume_profit_cents: import_cached(
                db,
                &id.metric_name("transfer_volume_in_profit_cumulative_cents"),
                version,
                indexes,
            )?,
            volume_loss_cents: import_cached(
                db,
                &id.metric_name("transfer_volume_in_loss_cumulative_cents"),
                version,
                indexes,
            )?,
            cdd: import_cached(
                db,
                &id.metric_name("coindays_destroyed_cumulative"),
                version,
                indexes,
            )?,
            cap: import_cached(db, &id.metric_name("realized_cap_cents"), version, indexes)?,
            profit: import_cached(
                db,
                &id.metric_name("realized_profit_cumulative_cents"),
                version,
                indexes,
            )?,
            loss: import_cached(
                db,
                &id.metric_name("realized_loss_cumulative_cents"),
                version,
                indexes,
            )?,
            value_destroyed: import_cached(
                db,
                &id.metric_name("value_destroyed_cumulative_cents"),
                version,
                indexes,
            )?,
            unrealized_profit: import_cached(
                db,
                &id.metric_name("unrealized_profit_cents"),
                version,
                indexes,
            )?,
            unrealized_loss: import_cached(
                db,
                &id.metric_name("unrealized_loss_cents"),
                version,
                indexes,
            )?,
            supply_profit: import_cached(
                db,
                &id.metric_name("supply_in_profit_sats"),
                version,
                indexes,
            )?,
            price: import_cached(
                db,
                &id.metric_name("cost_basis_by_coin_avg_cents"),
                version,
                indexes,
            )?,
            capitalized_price: import_cached(
                db,
                &id.metric_name("cost_basis_by_dollar_avg_cents"),
                version,
                indexes,
            )?,
            peak_regret: import_cached(
                db,
                &id.metric_name("realized_peak_regret_cumulative_cents"),
                version,
                indexes,
            )?,
            nupl: import_cached(db, &id.metric_name("nupl_ppm"), version + Version::ONE, indexes)?,
            cap_profit: import_cached(
                db,
                &id.metric_name("realized_cap_in_profit_cents"),
                version,
                indexes,
            )?,
            cap_loss: import_cached(
                db,
                &id.metric_name("realized_cap_in_loss_cents"),
                version,
                indexes,
            )?,
            peak: Default::default(),
        })
    }
    pub fn push(&mut self, d: &AccountingTotals, spot: Cents) {
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
        self.volume_loss_cents.push(d.volume_loss_cents);
        self.cdd.push(d.cdd);
        // From the exact sum, rounded once (the bands' rounded caps would add their errors).
        self.cap.push(d.cap_raw.to_cents());
        self.profit.push(d.profit);
        self.loss.push(d.loss);
        self.value_destroyed.push(d.value_destroyed);
        self.unrealized_profit.push(d.unrealized_profit);
        self.unrealized_loss.push(d.unrealized_loss);
        self.supply_profit.push(d.supply_profit);
        self.price.push(d.cap_raw.realized_price(d.supply));
        self.capitalized_price
            .push(d.capitalized_cap_raw.capitalized_price(d.cap_raw));
        self.peak_regret.push(peak);
        let net_pnl = CentsSigned::from_difference(d.unrealized_profit, d.unrealized_loss);
        self.nupl
            .push(Quotient::apply(net_pnl, d.supply.value_at(spot)));
        // The bands' capitals, each rounded once from its exact sum.
        self.cap_profit.push(d.capital_profit);
        self.cap_loss.push(d.capital_loss);
    }
}
