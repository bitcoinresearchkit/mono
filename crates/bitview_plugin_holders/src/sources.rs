use crate::data::Data;
use bitview_cohort::{AgeAggregate, AgeRangeId};
use bitview_plugin_age::Vecs as Age;
use brk_error::{Error, Result};
use brk_types::Height;
use vecdb::{AnyVec, ReadableVec, VecValue};

/// Calls `$column!(field, source)` for every source column of the age range `$r`, in read
/// order: the [`Data`] field it fills and the vector it reads.
macro_rules! for_each_column {
    ($r:ident, $column:ident) => {
        $column!(supply, &$r.supply.total.stored);
        $column!(count, &$r.outputs.unspent_count.stored);
        $column!(
            spent_count,
            $r.outputs.spent_count.stored.cumulative_source()
        );
        $column!(
            volume_sats,
            $r.activity.transfer_volume.sats.cumulative_source()
        );
        $column!(
            volume_cents,
            $r.activity.transfer_volume.cents.cumulative_source()
        );
        $column!(
            volume_profit_sats,
            $r.transfer_volume_in_profit.sats.cumulative_source()
        );
        $column!(
            volume_profit_cents,
            $r.transfer_volume_in_profit.cents.cumulative_source()
        );
        $column!(
            volume_loss_sats,
            $r.transfer_volume_in_loss.sats.cumulative_source()
        );
        $column!(
            volume_loss_cents,
            $r.transfer_volume_in_loss.cents.cumulative_source()
        );
        $column!(
            cdd,
            $r.activity.coindays_destroyed.stored.cumulative_source()
        );
        $column!(cap, &$r.capital.stored);
        $column!(cap_raw, &$r.raw.cap);
        $column!(capitalized_cap_raw, &$r.raw.capitalized_cap);
        $column!(profit, $r.realized.profit.stored.cumulative_source());
        $column!(loss, $r.realized.loss.stored.cumulative_source());
        $column!(net_pnl, $r.realized.net_pnl.stored.cumulative_source());
        $column!(
            value_destroyed,
            $r.realized.value_destroyed.stored.cumulative_source()
        );
        $column!(unrealized_profit, &$r.unrealized.profit.stored);
        $column!(unrealized_loss, &$r.unrealized.loss.stored);
        $column!(supply_profit, &$r.supply.in_profit.stored);
        $column!(supply_loss, &$r.supply.in_loss.stored);
        $column!(capital_profit, &$r.capital.in_profit.stored);
        $column!(capital_loss, &$r.capital.in_loss.stored);
        $column!(peak_regret_raw, &$r.raw.peak_regret);
    };
}

/// One batch of sums, constructed in a single walk over disjoint bands.
pub(crate) struct Sources<'a> {
    age: &'a Age,
}
impl<'a> Sources<'a> {
    pub fn new(age: &'a Age) -> Self {
        Self { age }
    }
    /// Every input vector, for the computation's version and length.
    pub fn vectors(&self) -> impl Iterator<Item = &dyn AnyVec> {
        AgeRangeId::ALL.iter().flat_map(move |id| {
            let r = id.select(&self.age.ranges);
            let mut vectors = Vec::new();
            macro_rules! push {
                ($field:ident, $source:expr) => {
                    vectors.push($source as &dyn AnyVec)
                };
            }
            for_each_column!(r, push);
            vectors
        })
    }
    pub fn collect(&self, start: usize, end: usize) -> Result<Vec<AgeAggregate<Data>>> {
        let mut result = vec![AgeAggregate::<Data>::default(); end - start];
        let mut band = vec![Data::default(); end - start];
        macro_rules! read_into {
            ($field:ident, $source:expr) => {
                read($source, start, &mut band, |row, value| row.$field = value)?
            };
        }
        for &id in AgeRangeId::ALL {
            let r = id.select(&self.age.ranges);
            band.fill(Data::default());
            for_each_column!(r, read_into);
            if id != AgeRangeId::Under1H {
                for row in &mut band {
                    row.adjusted_volume = row.volume_cents;
                    row.adjusted_value_destroyed = row.value_destroyed;
                }
            }
            for (target, &row) in result.iter_mut().zip(&band) {
                for entry in target.containing_mut(id) {
                    *entry += row;
                }
            }
        }
        Ok(result)
    }
}

/// Assigns `source`'s values from `start` into `rows`, read in place.
fn read<T: VecValue>(
    source: &impl ReadableVec<Height, T>,
    start: usize,
    rows: &mut [Data],
    mut assign: impl FnMut(&mut Data, T),
) -> Result<()> {
    let mut rows_left = rows.iter_mut();
    source.for_each_range_at(start, start + rows_left.len(), |value| {
        if let Some(row) = rows_left.next() {
            assign(row, value);
        }
    });
    if rows_left.len() != 0 {
        return Err(Error::NotFound(format!(
            "incomplete aggregate input {}",
            source.name()
        )));
    }
    Ok(())
}
