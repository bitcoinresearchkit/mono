use std::array;

use bitview_cohort::{AGE_RANGE_COUNT, Age, AgeRange, AgeRangeId, for_each_age_crossing};
use bitview_primitives::CentsCompact;
use brk_error::{Error, Result};
use brk_types::{Cents, Timestamp};
use rustc_hash::FxHashMap;
use statedb::{Cursor, State};

use crate::{COST_BASIS_PRICE_DIGITS, ProjectedBucket, projection::Projection};

/// A price/age view of reconstructed history. Queries build it for one block;
/// sequential consumers retain it and apply diffs without rebuilding it. Origin
/// amounts remain owned by statedb; this stores only their price/age grouping.
pub struct OriginUrpd {
    prices: Vec<(CentsCompact, u32)>,
    /// Supply per age range, one column over the price slots: neighbouring slots share their
    /// few occupied ages, so a projection reads mostly contiguous memory.
    amounts: [Vec<u64>; AGE_RANGE_COUNT],
    /// Nonzero ages per stable slot; four bytes avoid scanning 23 cells per bucket.
    occupied: Vec<u32>,
    origin_buckets: Vec<u32>,
    crossings: [usize; AGE_RANGE_COUNT - 1],
    reorder_at: usize,
}
impl OriginUrpd {
    pub fn new(state: &State, prices: &[Cents], timestamps: &[Timestamp]) -> Result<Self> {
        if state.len() > prices.len()
            || prices.len() != timestamps.len()
            || prices.iter().any(|p| p.is_nan())
        {
            return Err(Error::Internal("invalid origin price/timestamp coverage"));
        }
        let rounded = |p: Cents| CentsCompact::from(p).round_to_dollar(COST_BASIS_PRICE_DIGITS);
        let mut slots = FxHashMap::default();
        let origin_buckets: Vec<_> = prices
            .iter()
            .map(|&price| {
                let next = slots.len() as u32;
                *slots.entry(rounded(price)).or_insert(next)
            })
            .collect();
        let mut sorted: Vec<_> = slots.into_iter().collect();
        sorted.sort_unstable_by_key(|&(price, _)| price);
        let mut amounts: [Vec<u64>; AGE_RANGE_COUNT] = array::from_fn(|_| vec![0; sorted.len()]);
        if let Some(&current) = timestamps.get(state.len().wrapping_sub(1)) {
            for (h, amount) in state.amounts().iter().enumerate() {
                if amount.sats == 0 {
                    continue;
                }
                let age = AgeRangeId::from(Age::new(current, timestamps[h]));
                amounts[age.index()][origin_buckets[h] as usize] += amount.sats;
            }
        }
        let occupied = (0..sorted.len())
            .map(|slot| {
                amounts.iter().enumerate().fold(0, |mask, (age, column)| {
                    mask | (u32::from(column[slot] != 0) << age)
                })
            })
            .collect();
        let mut source = Self {
            prices: sorted,
            amounts,
            occupied,
            origin_buckets,
            crossings: [0; AGE_RANGE_COUNT - 1],
            reorder_at: 0,
        };
        source.reorder_slots();
        Ok(source)
    }
    /// Append newly available creation prices without rebuilding the occupied histogram.
    pub(crate) fn extend_prices(&mut self, prices: &[Cents]) -> Result<()> {
        if prices.iter().any(|p| p.is_nan()) {
            return Err(Error::Internal("invalid origin price"));
        }
        for &price in prices {
            let price = CentsCompact::from(price).round_to_dollar(COST_BASIS_PRICE_DIGITS);
            let slot = match self
                .prices
                .binary_search_by_key(&price, |&(price, _)| price)
            {
                Ok(position) => self.prices[position].1,
                Err(position) => {
                    let slot = self.occupied.len() as u32;
                    self.prices.insert(position, (price, slot));
                    for column in &mut self.amounts {
                        column.push(0);
                    }
                    self.occupied.push(0);
                    slot
                }
            };
            self.origin_buckets.push(slot);
        }
        if self.occupied.len() >= self.reorder_at {
            self.reorder_slots();
        }
        Ok(())
    }
    /// Keep slots in scan order across every age column, amortizing remaps over bucket doublings.
    fn reorder_slots(&mut self) {
        let len = self.prices.len();
        let mut slots = vec![0; len];
        for (index, (_, slot)) in self.prices.iter_mut().enumerate() {
            slots[*slot as usize] = index as u32;
            *slot = index as u32;
        }
        for slot in &mut self.origin_buckets {
            *slot = slots[*slot as usize];
        }
        // Origins now use the new slots; consume the map to reorder every column in place.
        for index in 0..len {
            while slots[index] as usize != index {
                let target = slots[index] as usize;
                for column in &mut self.amounts {
                    column.swap(index, target);
                }
                self.occupied.swap(index, target);
                slots.swap(index, target);
            }
        }
        self.reorder_at = len.max(1).saturating_mul(2);
    }
    /// Apply exactly one block, including all age crossings before its spends.
    pub(crate) fn advance(
        &mut self,
        cursor: &mut Cursor<'_>,
        timestamps: &[Timestamp],
    ) -> Result<()> {
        let h = cursor.state().len();
        let current = *timestamps
            .get(h)
            .ok_or(Error::Internal("missing origin timestamp"))?;
        if h >= self.origin_buckets.len() {
            return Err(Error::Internal("missing origin price"));
        }
        for_each_age_crossing(
            &timestamps[..h],
            current,
            &mut self.crossings,
            |origin, younger, older, _| {
                let sats = cursor.state().amounts()[origin].sats;
                if sats != 0 {
                    let slot = self.origin_buckets[origin] as usize;
                    self.amounts[younger.index()][slot] -= sats;
                    self.amounts[older.index()][slot] += sats;
                    if self.amounts[younger.index()][slot] == 0 {
                        self.occupied[slot] &= !(1 << younger.index());
                    }
                    self.occupied[slot] |= 1 << older.index();
                }
            },
        );
        let diff = cursor
            .advance()?
            .ok_or(Error::Internal("incomplete origin history"))?;
        let slot = self.origin_buckets[h] as usize;
        self.amounts[AgeRangeId::Under1H.index()][slot] += diff.created.sats;
        if diff.created.sats != 0 {
            self.occupied[slot] |= 1 << AgeRangeId::Under1H.index();
        }
        for (origin, amount) in diff.removed() {
            if amount.sats == 0 {
                continue;
            }
            let origin = origin as usize;
            let age = AgeRangeId::from(Age::new(current, timestamps[origin]));
            let slot = self.origin_buckets[origin] as usize;
            self.amounts[age.index()][slot] -= amount.sats;
            if self.amounts[age.index()][slot] == 0 {
                self.occupied[slot] &= !(1 << age.index());
            }
        }
        Ok(())
    }
    fn price_slots(&self) -> impl Iterator<Item = (CentsCompact, u32)> {
        self.prices.iter().copied()
    }
    /// Project all requested weights together, visiting each bucket's ages once.
    /// The caller can consume this view directly or retain its rows for repeated reads.
    /// Weighted entries retain input order; missing weights produce zero. Rounding
    /// happens after combining all ages in each price bucket.
    pub fn project<'a, const N: usize, const C: usize>(
        &'a self,
        weights: &[Option<&AgeRange<f64>>; N],
        cohorts: [&[AgeRangeId]; C],
    ) -> impl Iterator<Item = ProjectedBucket<N, C>> + 'a + use<'a, N, C> {
        let projection = Projection::new(weights, cohorts);
        self.price_slots().filter_map(move |(price, slot)| {
            let slot = slot as usize;
            projection.bucket(price, &self.amounts, slot, self.occupied[slot])
        })
    }
}
