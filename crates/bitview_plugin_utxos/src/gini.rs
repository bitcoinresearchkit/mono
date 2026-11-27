use std::iter::once;

use bitview_cohort::AmountRange;
use bitview_compute::{prepare_computed, write_locked};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Day1, Float};
use bitview_traversable::Traversable;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{
    AnyVec, Database, ImportableVec, PcoVec, ReadableVec, Rw, StorageMode, StoredVec, VecIndex,
    WritableVec,
};

use crate::metrics::{CohortVecs, ZeroVecs};

/// The amount bands above zero, ascending.
const BANDS: usize = 14;

/// The UTXO set's inequality, read at each day's close (the day's last block, UTC). Stored rather
/// than composed: the formula reads 29 band series per request. Each day's value is the bands at
/// its last block alone, so a rewrite recomputes the days it touches and nothing is kept.
#[derive(Traversable)]
pub struct AmountGini<M: StorageMode = Rw> {
    /// Gini coefficient (Corrado Gini, 1912) of the unspent outputs' amounts, zero-value outputs
    /// included. Read from the amount bands, each band's outputs taken as holding equal shares of
    /// its supply, so it is a floor under the coefficient of the outputs one by one. Null without
    /// outputs.
    #[traversable(
        lead = "Gini coefficient of UTXO amounts: 0 when every unspent output holds the same, toward 1 as a few hold it all."
    )]
    amount_gini: M::Stored<PcoVec<Day1, Float>>,
}

impl AmountGini {
    pub(crate) fn import(db: &Database, version: Version) -> Result<Self> {
        Ok(Self {
            amount_gini: PcoVec::import(db, "utxo_amount_gini", version)?,
        })
    }

    /// From the day before block `from`'s on: the first block a rewrite replaces can fall on a
    /// later day than the one it removed.
    pub(crate) fn compute(
        &mut self,
        amounts: &AmountRange<CohortVecs, ZeroVecs>,
        mappings: &MappingsVecs,
        from: Height,
        exit: &Exit,
    ) -> Result<()> {
        let (zero, bands) = amounts.split();
        let counts = once(zero.utxos.count.stored.read_only_clone())
            .chain(bands.iter().map(|b| b.utxos.count.stored.read_only_clone()))
            .collect::<Vec<_>>();
        let supplies = bands
            .iter()
            .map(|b| b.supply.total.stored.read_only_clone())
            .collect::<Vec<_>>();
        let day_of_height = &mappings.height.day1;
        let first_height = &mappings.day1.first_height;
        let end = counts[0].len();
        debug_assert!(
            counts
                .iter()
                .map(AnyVec::len)
                .chain(supplies.iter().map(AnyVec::len))
                .all(|len| len == end),
            "the amount bands are written together"
        );
        if end == 0 {
            return Ok(());
        }
        let version = Version::ONE
            + day_of_height.version()
            + first_height.version()
            + counts.iter().map(AnyVec::version).sum::<Version>()
            + supplies.iter().map(AnyVec::version).sum::<Version>();
        let last = Height::from(end - 1);
        let last_day = day_of_height.collect_one(last).data()?.to_usize();
        // From the last day written at the latest: it was written open, and a stop between the
        // bands crossing its midnight and this update would otherwise leave it at an older tip.
        let from_day = day_of_height
            .collect_one(from.decremented().unwrap_or(Height::ZERO).min(last))
            .data()?
            .to_usize()
            .min(self.amount_gini.len().saturating_sub(1));
        let first_day = prepare_computed([&mut self.amount_gini], version, from_day, exit)?;

        let start_of = |day: usize| {
            first_height
                .collect_one(Day1::from(day))
                .map_or(end, usize::from)
                .min(end)
        };
        // Each day's last block; a day without blocks (January 2009) has none.
        let mut closes = Vec::with_capacity(last_day + 1 - first_day);
        let mut start = start_of(first_day);
        for day in first_day..=last_day {
            let next = if day == last_day {
                end
            } else {
                start_of(day + 1)
            };
            closes.push((next > start).then(|| next - 1));
            start = next;
        }
        let heights = closes.iter().flatten().copied().collect::<Vec<_>>();
        let counts = counts
            .iter()
            .map(|v| v.read_sorted_at(&heights))
            .collect::<Vec<_>>();
        let supplies = supplies
            .iter()
            .map(|v| v.read_sorted_at(&heights))
            .collect::<Vec<_>>();
        let mut values = (0..heights.len()).map(|at| {
            gini(
                std::array::from_fn(|k| u64::from(counts[k][at]) as f64),
                std::array::from_fn(|k| u64::from(supplies[k][at]) as f64),
            )
        });
        for close in closes {
            let value = close.map_or(f64::NAN, |_| values.next().unwrap());
            self.amount_gini.push(Float::from(value));
        }
        write_locked([&mut self.amount_gini], exit)
    }
}

/// The Gini coefficient of `counts[k]` outputs each holding an equal share of `supplies[k - 1]`,
/// band 0 holding nothing and the rest ascending: one minus twice the area under the Lorenz
/// curve, the bands its trapezoids. NaN without outputs or value.
fn gini(counts: [f64; BANDS + 1], supplies: [f64; BANDS]) -> f64 {
    let outputs = counts.iter().sum::<f64>();
    let value = supplies.iter().sum::<f64>();
    let mut below = 0.0;
    let mut area = 0.0;
    for (n, s) in counts[1..].iter().zip(supplies) {
        area += n * (2.0 * below + s);
        below += s;
    }
    if outputs == 0.0 || value == 0.0 {
        f64::NAN
    } else {
        1.0 - area / (outputs * value)
    }
}
