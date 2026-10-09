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
        let c = &self.age.cohorts;
        AgeRangeId::ALL.iter().flat_map(move |id| {
            let vectors: [&dyn AnyVec; 24] = [
                c.supply.total.stored.get(id.cohort()).unwrap(),
                c.outputs.unspent_count.stored.get(id.cohort()).unwrap(),
                c.outputs
                    .spent_count
                    .stored
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.activity
                    .transfer_volume
                    .stored
                    .sats
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.activity
                    .transfer_volume
                    .stored
                    .cents
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.activity
                    .transfer_volume_in_profit
                    .stored
                    .sats
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.activity
                    .transfer_volume_in_profit
                    .stored
                    .cents
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.activity
                    .transfer_volume_in_loss
                    .stored
                    .sats
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.activity
                    .transfer_volume_in_loss
                    .stored
                    .cents
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.activity
                    .coindays_destroyed
                    .stored
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.realized.cap.stored.get(id.cohort()).unwrap(),
                id.select(&c.realized.cap_raw.age),
                id.select(&c.realized.capitalized_cap_raw.age),
                c.realized.profit.stored.stored.get(id.cohort()).unwrap(),
                c.realized.loss.stored.stored.get(id.cohort()).unwrap(),
                c.realized.net_pnl.stored.stored.get(id.cohort()).unwrap(),
                c.realized
                    .value_destroyed
                    .stored
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                c.unrealized.profit.stored.get(id.cohort()).unwrap(),
                c.unrealized.loss.stored.get(id.cohort()).unwrap(),
                c.supply.in_profit.stored.get(id.cohort()).unwrap(),
                c.supply.in_loss.stored.get(id.cohort()).unwrap(),
                id.select(&c.unrealized.capitalized_cap_in_profit_raw.age),
                id.select(&c.unrealized.capitalized_cap_in_loss_raw.age),
                id.select(&c.realized.peak_regret_raw.age),
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
        let c = &self.age.cohorts;
        for &id in AgeRangeId::ALL {
            let mut band = vec![Data::default(); end - start];
            read(
                c.supply.total.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.supply = value,
            )?;
            read(
                c.outputs.unspent_count.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.count = value,
            )?;
            read(
                c.outputs
                    .spent_count
                    .stored
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.spent_count = value,
            )?;
            read(
                c.activity
                    .transfer_volume
                    .stored
                    .sats
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.volume_sats = value,
            )?;
            read(
                c.activity
                    .transfer_volume
                    .stored
                    .cents
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.volume_cents = value,
            )?;
            read(
                c.activity
                    .transfer_volume_in_profit
                    .stored
                    .sats
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.volume_profit_sats = value,
            )?;
            read(
                c.activity
                    .transfer_volume_in_profit
                    .stored
                    .cents
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.volume_profit_cents = value,
            )?;
            read(
                c.activity
                    .transfer_volume_in_loss
                    .stored
                    .sats
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.volume_loss_sats = value,
            )?;
            read(
                c.activity
                    .transfer_volume_in_loss
                    .stored
                    .cents
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.volume_loss_cents = value,
            )?;
            read(
                c.activity
                    .coindays_destroyed
                    .stored
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.cdd = value,
            )?;
            read(
                c.realized.cap.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.cap = value,
            )?;
            read(
                id.select(&c.realized.cap_raw.age),
                start,
                &mut band,
                |row, value| row.cap_raw = value,
            )?;
            read(
                id.select(&c.realized.capitalized_cap_raw.age),
                start,
                &mut band,
                |row, value| row.capitalized_cap_raw = value,
            )?;
            read(
                c.realized.profit.stored.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.profit = value,
            )?;
            read(
                c.realized.loss.stored.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.loss = value,
            )?;
            read(
                c.realized.net_pnl.stored.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.net_pnl = value,
            )?;
            read(
                c.realized
                    .value_destroyed
                    .stored
                    .stored
                    .get(id.cohort())
                    .unwrap(),
                start,
                &mut band,
                |row, value| row.value_destroyed = value,
            )?;
            read(
                c.unrealized.profit.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.unrealized_profit = value,
            )?;
            read(
                c.unrealized.loss.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.unrealized_loss = value,
            )?;
            read(
                c.supply.in_profit.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.supply_profit = value,
            )?;
            read(
                c.supply.in_loss.stored.get(id.cohort()).unwrap(),
                start,
                &mut band,
                |row, value| row.supply_loss = value,
            )?;
            read(
                id.select(&c.unrealized.capitalized_cap_in_profit_raw.age),
                start,
                &mut band,
                |row, value| row.capitalized_profit = value,
            )?;
            read(
                id.select(&c.unrealized.capitalized_cap_in_loss_raw.age),
                start,
                &mut band,
                |row, value| row.capitalized_loss = value,
            )?;
            read(
                id.select(&c.realized.peak_regret_raw.age),
                start,
                &mut band,
                |row, value| row.peak_regret_raw = value,
            )?;
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
