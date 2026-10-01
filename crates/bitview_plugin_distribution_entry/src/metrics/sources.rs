use bitview_cohort::{CohortContext, CohortId};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, import_cached};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Height, Sats, StoredF64, StoredU64, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::CumulativeSource;

#[derive(Traversable)]
pub(crate) struct Sources<M: StorageMode = Rw> {
    pub supply: CachedSeries<Height, Sats, M>,
    pub supply_in_profit: CachedSeries<Height, Sats, M>,
    pub supply_in_loss: CachedSeries<Height, Sats, M>,
    pub unspent_count: CachedSeries<Height, StoredU64, M>,
    pub realized_cap: CachedSeries<Height, Cents, M>,
    pub realized_price: CachedSeries<Height, Cents, M>,
    pub unrealized_profit: CachedSeries<Height, Cents, M>,
    pub unrealized_loss: CachedSeries<Height, Cents, M>,
    pub unrealized_net_pnl: CachedSeries<Height, CentsSigned, M>,
    pub spent_count: CumulativeSource<StoredU64, M>,
    pub transfer_sats: CumulativeSource<Sats, M>,
    pub transfer_cents: CumulativeSource<Cents, M>,
    pub profit_sats: CumulativeSource<Sats, M>,
    pub profit_cents: CumulativeSource<Cents, M>,
    pub loss_sats: CumulativeSource<Sats, M>,
    pub loss_cents: CumulativeSource<Cents, M>,
    pub coindays: CumulativeSource<StoredF64, M>,
    pub realized_profit: CumulativeSource<Cents, M>,
    pub realized_loss: CumulativeSource<Cents, M>,
    pub realized_net_pnl: CumulativeSource<CentsSigned, M>,
    pub value_destroyed: CumulativeSource<Cents, M>,
}

impl Sources {
    pub(crate) fn import(db: &Database, id: CohortId, version: Version) -> Result<Self> {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        Ok(Self {
            supply: import_cached(db, &name("supply_sats"), version)?,
            supply_in_profit: import_cached(db, &name("supply_in_profit_sats"), version)?,
            supply_in_loss: import_cached(db, &name("supply_in_loss_sats"), version)?,
            unspent_count: import_cached(db, &name("utxo_count"), version)?,
            realized_cap: import_cached(db, &name("realized_cap_cents"), version)?,
            realized_price: import_cached(db, &name("realized_price_cents"), version)?,
            unrealized_profit: import_cached(db, &name("unrealized_profit_cents"), version)?,
            unrealized_loss: import_cached(db, &name("unrealized_loss_cents"), version)?,
            unrealized_net_pnl: import_cached(db, &name("net_unrealized_pnl_cents"), version)?,
            spent_count: CumulativeSource::import(
                db,
                &name("spent_utxo_count_cumulative"),
                version,
            )?,
            transfer_sats: CumulativeSource::import(
                db,
                &name("transfer_volume_cumulative_sats"),
                version,
            )?,
            transfer_cents: CumulativeSource::import(
                db,
                &name("transfer_volume_cumulative_cents"),
                version,
            )?,
            profit_sats: CumulativeSource::import(
                db,
                &name("transfer_volume_in_profit_cumulative_sats"),
                version,
            )?,
            profit_cents: CumulativeSource::import(
                db,
                &name("transfer_volume_in_profit_cumulative_cents"),
                version,
            )?,
            loss_sats: CumulativeSource::import(
                db,
                &name("transfer_volume_in_loss_cumulative_sats"),
                version,
            )?,
            loss_cents: CumulativeSource::import(
                db,
                &name("transfer_volume_in_loss_cumulative_cents"),
                version,
            )?,
            coindays: CumulativeSource::import(
                db,
                &name("coindays_destroyed_cumulative"),
                version,
            )?,
            realized_profit: CumulativeSource::import(
                db,
                &name("realized_profit_cumulative_cents"),
                version,
            )?,
            realized_loss: CumulativeSource::import(
                db,
                &name("realized_loss_cumulative_cents"),
                version,
            )?,
            realized_net_pnl: CumulativeSource::import(
                db,
                &name("net_realized_pnl_cumulative_cents"),
                version,
            )?,
            value_destroyed: CumulativeSource::import(
                db,
                &name("value_destroyed_cumulative_cents"),
                version,
            )?,
        })
    }

    pub fn min_len(&self) -> usize {
        self.iter_any_exportable()
            .map(|vec| vec.len())
            .min()
            .unwrap_or_default()
    }

    pub fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        vec![
            &mut self.supply,
            &mut self.supply_in_profit,
            &mut self.supply_in_loss,
            &mut self.unspent_count,
            &mut self.realized_cap,
            &mut self.realized_price,
            &mut self.unrealized_profit,
            &mut self.unrealized_loss,
            &mut self.unrealized_net_pnl,
            self.spent_count.stored_mut(),
            self.transfer_sats.stored_mut(),
            self.transfer_cents.stored_mut(),
            self.profit_sats.stored_mut(),
            self.profit_cents.stored_mut(),
            self.loss_sats.stored_mut(),
            self.loss_cents.stored_mut(),
            self.coindays.stored_mut(),
            self.realized_profit.stored_mut(),
            self.realized_loss.stored_mut(),
            self.realized_net_pnl.stored_mut(),
            self.value_destroyed.stored_mut(),
        ]
    }
}
