use crate::data::Data;
use bitview_cohort::{AgeAggregate, AgeRangeId};
use bitview_plugin_age::Vecs as Age;
use brk_error::{Error, Result};
use brk_types::{Height, Version};
use vecdb::{AnyVec, ReadableVec, VecValue};

/// One batch of sums, constructed in a single walk over disjoint bands.
pub(crate) struct Sources<'a> {
    age: &'a Age,
}
impl<'a> Sources<'a> {
    pub fn new(age: &'a Age) -> Self {
        Self { age }
    }
    fn vectors(&self) -> impl Iterator<Item = &dyn AnyVec> {
        AgeRangeId::ALL.iter().flat_map(move |id| {
            let r = id.select(&self.age.ranges);
            let vectors: [&dyn AnyVec; 22] = [
                &r.supply.total.stored,
                &r.outputs.unspent_count.stored,
                r.outputs.spent_count.stored.cumulative_source(),
                r.activity.transfer_volume.sats.cumulative_source(),
                r.activity.transfer_volume.cents.cumulative_source(),
                r.activity
                    .transfer_volume_in_profit
                    .sats
                    .cumulative_source(),
                r.activity
                    .transfer_volume_in_profit
                    .cents
                    .cumulative_source(),
                r.activity.transfer_volume_in_loss.sats.cumulative_source(),
                r.activity.transfer_volume_in_loss.cents.cumulative_source(),
                r.activity.coindays_destroyed.stored.cumulative_source(),
                &r.capital.stored,
                &r.raw.cap,
                &r.raw.capitalized_cap,
                r.realized.profit.stored.cumulative_source(),
                r.realized.loss.stored.cumulative_source(),
                r.realized.net_pnl.stored.cumulative_source(),
                r.realized.value_destroyed.stored.cumulative_source(),
                &r.unrealized.profit.stored,
                &r.unrealized.loss.stored,
                &r.supply.in_profit.stored,
                &r.supply.in_loss.stored,
                &r.raw.peak_regret,
            ];
            vectors
        })
    }
    pub fn version(&self) -> Version {
        Version::combine_all(self.vectors().map(AnyVec::version))
    }
    pub fn len(&self) -> usize {
        self.vectors().map(AnyVec::len).min().unwrap_or_default()
    }
    pub fn collect(&self, start: usize, end: usize) -> Result<Vec<AgeAggregate<Data>>> {
        let mut result = vec![AgeAggregate::<Data>::default(); end - start];
        for &id in AgeRangeId::ALL {
            let r = id.select(&self.age.ranges);
            let mut band = vec![Data::default(); end - start];
            read(&r.supply.total.stored, start, &mut band, |row, value| {
                row.supply = value
            })?;
            read(
                &r.outputs.unspent_count.stored,
                start,
                &mut band,
                |row, value| row.count = value,
            )?;
            read(
                r.outputs.spent_count.stored.cumulative_source(),
                start,
                &mut band,
                |row, value| row.spent_count = value,
            )?;
            read(
                r.activity.transfer_volume.sats.cumulative_source(),
                start,
                &mut band,
                |row, value| row.volume_sats = value,
            )?;
            read(
                r.activity.transfer_volume.cents.cumulative_source(),
                start,
                &mut band,
                |row, value| row.volume_cents = value,
            )?;
            read(
                r.activity
                    .transfer_volume_in_profit
                    .sats
                    .cumulative_source(),
                start,
                &mut band,
                |row, value| row.volume_profit_sats = value,
            )?;
            read(
                r.activity
                    .transfer_volume_in_profit
                    .cents
                    .cumulative_source(),
                start,
                &mut band,
                |row, value| row.volume_profit_cents = value,
            )?;
            read(
                r.activity.transfer_volume_in_loss.sats.cumulative_source(),
                start,
                &mut band,
                |row, value| row.volume_loss_sats = value,
            )?;
            read(
                r.activity.transfer_volume_in_loss.cents.cumulative_source(),
                start,
                &mut band,
                |row, value| row.volume_loss_cents = value,
            )?;
            read(
                r.activity.coindays_destroyed.stored.cumulative_source(),
                start,
                &mut band,
                |row, value| row.cdd = value,
            )?;
            read(&r.capital.stored, start, &mut band, |row, value| {
                row.cap = value
            })?;
            read(&r.raw.cap, start, &mut band, |row, value| {
                row.cap_raw = value
            })?;
            read(&r.raw.capitalized_cap, start, &mut band, |row, value| {
                row.capitalized_cap_raw = value
            })?;
            read(
                r.realized.profit.stored.cumulative_source(),
                start,
                &mut band,
                |row, value| row.profit = value,
            )?;
            read(
                r.realized.loss.stored.cumulative_source(),
                start,
                &mut band,
                |row, value| row.loss = value,
            )?;
            read(
                r.realized.net_pnl.stored.cumulative_source(),
                start,
                &mut band,
                |row, value| row.net_pnl = value,
            )?;
            read(
                r.realized.value_destroyed.stored.cumulative_source(),
                start,
                &mut band,
                |row, value| row.value_destroyed = value,
            )?;
            read(
                &r.unrealized.profit.stored,
                start,
                &mut band,
                |row, value| row.unrealized_profit = value,
            )?;
            read(&r.unrealized.loss.stored, start, &mut band, |row, value| {
                row.unrealized_loss = value
            })?;
            read(
                &r.supply.in_profit.stored,
                start,
                &mut band,
                |row, value| row.supply_profit = value,
            )?;
            read(&r.supply.in_loss.stored, start, &mut band, |row, value| {
                row.supply_loss = value
            })?;
            read(&r.raw.peak_regret, start, &mut band, |row, value| {
                row.peak_regret_raw = value
            })?;
            if id != AgeRangeId::Under1H {
                for row in &mut band {
                    row.adjusted_volume = row.volume_cents;
                    row.adjusted_value_destroyed = row.value_destroyed;
                }
            }
            for (target, row) in result.iter_mut().zip(band) {
                for entry in target.containing_mut(id) {
                    *entry += row;
                }
            }
        }
        Ok(result)
    }
}

fn read<T: VecValue>(
    source: &impl ReadableVec<Height, T>,
    start: usize,
    rows: &mut [Data],
    mut assign: impl FnMut(&mut Data, T),
) -> Result<()> {
    let values = source.collect_range_at(start, start + rows.len());
    if values.len() != rows.len() {
        return Err(Error::NotFound(format!(
            "incomplete aggregate input {}",
            source.name()
        )));
    }
    for (row, value) in rows.iter_mut().zip(values) {
        assign(row, value);
    }
    Ok(())
}
