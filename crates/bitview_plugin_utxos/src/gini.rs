use bitview_cohort::AmountRange;
use bitview_compute::{day_closes, prepare_days, write_locked};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Day1, Float};
use bitview_traversable::Traversable;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, ImportableVec, PcoVec, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, Rw, StorageMode, StoredVec, VecIndex, WritableVec,
};

use crate::metrics::{CohortVecs, ZeroVecs};

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
    #[traversable(skip)]
    day_of_height: ReadableBoxedVec<Height, Day1>,
    #[traversable(skip)]
    first_height: ReadableBoxedVec<Day1, Height>,
}

impl AmountGini {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        Ok(Self {
            amount_gini: PcoVec::import(db, "utxo_amount_gini", version)?,
            day_of_height: mappings.height.day1.read_only_boxed_clone(),
            first_height: mappings.day1.first_height.read_only_boxed_clone(),
        })
    }

    /// Drops the days that read the bands from block `len` on: from the day of the block before it,
    /// whose close may have been one of them.
    pub(crate) fn truncate(&mut self, len: usize) -> Result<()> {
        let day = match len.checked_sub(1) {
            Some(last) => self.day_of_height.collect_one_at(last).data()?.to_usize(),
            None => 0,
        };
        self.amount_gini.any_truncate_if_needed_at(day)?;
        Ok(())
    }

    /// Writes the days from the one before block `from`'s on.
    pub(crate) fn compute(
        &mut self,
        amounts: &AmountRange<CohortVecs, ZeroVecs>,
        from: Height,
        exit: &Exit,
    ) -> Result<()> {
        let (zero, bands) = amounts.split();
        let zero_counts = zero.utxos.count.stored.read_only_clone();
        let counts = bands.map(|b| b.utxos.count.stored.read_only_clone());
        let supplies = bands.map(|b| b.supply.total.stored.read_only_clone());
        let end = zero_counts.len();
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
            + self.day_of_height.version()
            + self.first_height.version()
            + zero_counts.version()
            + counts.iter().map(AnyVec::version).sum::<Version>()
            + supplies.iter().map(AnyVec::version).sum::<Version>();
        let last_day = self.day_of_height.collect_one_at(end - 1).data()?.to_usize();
        let first_day = prepare_days(
            [&mut self.amount_gini],
            version,
            &self.day_of_height,
            from,
            last_day + 1,
            exit,
        )?;

        let closes = day_closes(&self.first_height, first_day..last_day + 1, end);
        let heights = closes.iter().flatten().copied().collect::<Vec<_>>();
        let zero_counts = zero_counts.read_sorted_at(&heights);
        let counts = counts.map(|v| v.read_sorted_at(&heights));
        let supplies = supplies.map(|v| v.read_sorted_at(&heights));
        let mut values = (0..heights.len()).map(|at| {
            gini(
                u64::from(zero_counts[at]) as f64,
                counts.each_ref().map(|c| u64::from(c[at]) as f64),
                supplies.each_ref().map(|s| u64::from(s[at]) as f64),
            )
        });
        for close in closes {
            let value = close.map_or(f64::NAN, |_| values.next().unwrap());
            self.amount_gini.push(Float::from(value));
        }
        write_locked([&mut self.amount_gini], exit)
    }
}

/// The Gini coefficient of `zero` outputs holding nothing and, in each band above it, ascending,
/// `counts[k]` outputs each holding an equal share of `supplies[k]`: one minus twice the area
/// under the Lorenz curve, the bands its trapezoids. NaN without outputs or value.
fn gini<const N: usize>(zero: f64, counts: [f64; N], supplies: [f64; N]) -> f64 {
    let outputs = zero + counts.iter().sum::<f64>();
    let value = supplies.iter().sum::<f64>();
    let mut below = 0.0;
    let mut area = 0.0;
    for (n, s) in counts.into_iter().zip(supplies) {
        area += n * (2.0 * below + s);
        below += s;
    }
    if outputs == 0.0 || value == 0.0 {
        f64::NAN
    } else {
        1.0 - area / (outputs * value)
    }
}
