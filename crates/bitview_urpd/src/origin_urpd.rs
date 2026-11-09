use std::array;

use bitview_cohort::{AGE_BOUNDARIES, AGE_RANGE_COUNT, Age, AgeRange, AgeRangeId, for_each_age_crossing};
use bitview_primitives::CentsCompact;
use brk_error::{Error, Result};
use brk_types::{Cents, Timestamp};
use statedb::{Cursor, State};

use crate::{COST_BASIS_PRICE_DIGITS, ProjectedBucket, projection::Projection};

/// Prices below this many cents aren't rounded.
const EXACT_PRICES: u32 = 10u32.pow(COST_BASIS_PRICE_DIGITS);
/// The rounded prices a decade above those holds.
const PRICES_PER_DECADE: u32 = EXACT_PRICES - EXACT_PRICES / 10;
/// Every price rounding can produce, a place each.
const PRICE_PLACES: usize = (EXACT_PRICES + PRICES_PER_DECADE * (u32::MAX.ilog10() + 1 - COST_BASIS_PRICE_DIGITS)) as usize;

/// A rounded price's place among all the prices rounding can produce, in ascending price order.
#[inline]
fn price_place(rounded: CentsCompact) -> usize {
    let cents = rounded.inner();
    if cents < EXACT_PRICES {
        return cents as usize;
    }
    let decade = cents.ilog10() - COST_BASIS_PRICE_DIGITS;
    let leading = cents / 10u32.pow(decade + 1);
    (EXACT_PRICES + PRICES_PER_DECADE * decade + leading - EXACT_PRICES / 10) as usize
}

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
        debug_assert!(timestamps.is_sorted(), "origin timestamps must be monotonic");
        // Each place's price, found once: walking the places then gives the prices in order.
        let mut at_place = vec![CentsCompact::NAN; PRICE_PLACES];
        let mut origin_buckets: Vec<u32> = prices
            .iter()
            .map(|&price| {
                let rounded =
                    CentsCompact::from(price).round_to_significant(COST_BASIS_PRICE_DIGITS);
                let place = price_place(rounded);
                at_place[place] = rounded;
                place as u32
            })
            .collect();
        let mut sorted = Vec::new();
        let slots: Vec<u32> = at_place
            .iter()
            .map(|&price| {
                let slot = sorted.len() as u32;
                if !price.is_nan() {
                    sorted.push((price, slot));
                }
                slot
            })
            .collect();
        for bucket in &mut origin_buckets {
            *bucket = slots[*bucket as usize];
        }
        let mut amounts: [Vec<u64>; AGE_RANGE_COUNT] = array::from_fn(|_| vec![0; sorted.len()]);
        let len = state.len();
        if let Some(&current) = timestamps.get(len.wrapping_sub(1)) {
            // Older blocks first: an age range is a run of blocks, as timestamps never go down.
            let mut start = 0;
            for (age, column) in amounts.iter_mut().enumerate().rev() {
                let end = age.checked_sub(1).map_or(len, |younger| {
                    timestamps[..len]
                        .partition_point(|&t| current.hours_since(t) >= AGE_BOUNDARIES[younger])
                });
                for (amount, &slot) in state.amounts()[start..end]
                    .iter()
                    .zip(&origin_buckets[start..end])
                {
                    column[slot as usize] += amount.sats;
                }
                start = end;
            }
        }
        let occupied = (0..sorted.len())
            .map(|slot| {
                amounts.iter().enumerate().fold(0, |mask, (age, column)| {
                    mask | (u32::from(column[slot] != 0) << age)
                })
            })
            .collect();
        Ok(Self {
            reorder_at: sorted.len().max(1) * 2,
            prices: sorted,
            amounts,
            occupied,
            origin_buckets,
            crossings: [0; AGE_RANGE_COUNT - 1],
        })
    }
    /// Adds the next block's creation price without rebuilding the occupied histogram.
    pub(crate) fn push_price(&mut self, price: Cents) -> Result<()> {
        if price.is_nan() {
            return Err(Error::Internal("invalid origin price"));
        }
        let price = CentsCompact::from(price).round_to_significant(COST_BASIS_PRICE_DIGITS);
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
        self.reorder_at = len.max(1) * 2;
    }
    /// Apply exactly one block, including all age crossings before its spends.
    pub(crate) fn advance(
        &mut self,
        cursor: &mut Cursor<'_>,
        timestamps: &[Timestamp],
    ) -> Result<()> {
        // The replay checked its sources cover the range and pushed this block's price.
        let h = cursor.state().len();
        let current = timestamps[h];
        debug_assert!(h < self.origin_buckets.len(), "missing origin price");
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
            .expect("history covers the replayed range");
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
    /// Project all requested weights together, visiting each bucket's ages once.
    /// Weighted entries retain input order; missing weights produce zero. Rounding
    /// happens after combining all ages in each price bucket.
    pub fn project<'a, const N: usize, const C: usize>(
        &'a self,
        weights: &[Option<&AgeRange<f64>>; N],
        cohorts: [&[AgeRangeId]; C],
    ) -> impl Iterator<Item = ProjectedBucket<N, C>> + 'a + use<'a, N, C> {
        let projection = Projection::new(weights, cohorts);
        self.prices.iter().filter_map(move |&(price, slot)| {
            let slot = slot as usize;
            projection.bucket(price, &self.amounts, slot, self.occupied[slot])
        })
    }
}
