use std::array;

use bitview_cohort::{Age, AgeAggregateId, AgeRangeId};
use bitview_primitives::CentsCompact;
use brk_error::{Error, Result};
use brk_types::{Cents, Timestamp};

use super::{BucketedPrice, HeldExtremes, PriceIndex, PriceTotals};
use crate::walk::{Block, Origins};

/// The sats and capital held at each price under four age filters (all, short-term holders,
/// under four months, under six months), and the extremes of the origins holding coins: the
/// state of a walk consumer, which restores it from the origins and applies each block to it.
#[derive(Default)]
pub struct AgeIndex {
    pub index: PriceIndex<4>,
    pub held: HeldExtremes,
    /// Each block's price with its bucket, mapped once.
    prices: Vec<BucketedPrice>,
}

impl AgeIndex {
    pub fn restore(&mut self, origins: &Origins<'_>) -> Result<()> {
        self.prices.clear();
        self.bucket(origins.prices)?;
        self.index.reset();
        let amounts = origins.amounts;
        if let Some(last) = amounts.len().checked_sub(1) {
            let current = origins.timestamps[last];
            for (h, amount) in amounts.iter().enumerate() {
                if amount.sats != 0 {
                    self.index.add_raw(
                        self.prices[h],
                        amount.sats as i64,
                        filters(age(current, origins.timestamps[h])),
                    );
                }
            }
        }
        self.index.build();
        self.held.restore(amounts, &self.prices);
        Ok(())
    }

    pub fn apply(&mut self, block: &Block<'_>) -> Result<()> {
        self.bucket(block.prices)?;
        let Self {
            index,
            held,
            prices,
        } = self;
        // Sats leave the filters only their old age is in and join those only their new one is.
        for moved in block.moves() {
            let (from, to) = (
                moved.from.map_or([false; 4], filters),
                moved.to.map_or([false; 4], filters),
            );
            let mut add = |sats: i64, mask: [bool; 4]| {
                if mask.contains(&true) {
                    index.add(prices[moved.origin], sats, mask);
                }
            };
            add(-(moved.sats as i64), array::from_fn(|i| from[i] && !to[i]));
            add(moved.sats as i64, array::from_fn(|i| to[i] && !from[i]));
        }
        for (origin, _) in block.diff.removed() {
            held.spend_from(origin);
        }
        held.settle(block.height, block.amounts, prices);
        Ok(())
    }

    /// Maps the prices not yet mapped.
    fn bucket(&mut self, prices: &[Cents]) -> Result<()> {
        let new = &prices[self.prices.len().min(prices.len())..];
        if new.iter().any(|price| price.is_nan()) {
            return Err(Error::NotFound("invalid age-index price history".into()));
        }
        self.prices
            .extend(new.iter().map(|&price| BucketedPrice::from(CentsCompact::from(price))));
        self.held.reserve(self.prices.len());
        Ok(())
    }
}

fn age(current: Timestamp, origin: Timestamp) -> AgeRangeId {
    AgeRangeId::from(Age::new(current, origin))
}

fn filters(age: AgeRangeId) -> [bool; 4] {
    [
        true,
        AgeAggregateId::Sth.contains(age),
        AgeAggregateId::Under4M.contains(age),
        AgeAggregateId::Under6M.contains(age),
    ]
}

/// The index's filter of each age cutoff: 4, 5 and 6 months.
const CUTOFF_FILTERS: [usize; 3] = [2, 1, 3];

/// A cohort as an age cut: its cutoff and whether it's the young side of it; all of them, none.
fn cut_of(id: AgeAggregateId) -> Option<(usize, bool)> {
    match id {
        AgeAggregateId::All => None,
        AgeAggregateId::Under4M => Some((0, true)),
        AgeAggregateId::Over4M => Some((0, false)),
        AgeAggregateId::Sth => Some((1, true)),
        AgeAggregateId::Lth => Some((1, false)),
        AgeAggregateId::Under6M => Some((2, true)),
        AgeAggregateId::Over6M => Some((2, false)),
    }
}

/// A cohort's lowest and highest cost basis, exactly: of its origins still holding coins, the
/// lowest and highest price, at the state's last height (`len` origins). An age cut's young side
/// is its origins from the first its filter holds (the filter turns once: origins come in age
/// order), its old side those before.
pub fn extremes(
    extremes: &HeldExtremes,
    id: AgeAggregateId,
    timestamps: &[Timestamp],
    len: usize,
) -> (Cents, Cents) {
    let Some(last) = len.checked_sub(1) else {
        return (Cents::NAN, Cents::NAN);
    };
    let current = timestamps[last];
    let cut = |cutoff: usize| {
        timestamps[..len].partition_point(|&t| !filters(age(current, t))[CUTOFF_FILTERS[cutoff]])
    };
    let (from, to) = match cut_of(id) {
        None => (0, len),
        Some((cutoff, true)) => (cut(cutoff), len),
        Some((cutoff, false)) => (0, cut(cutoff)),
    };
    extremes.range(from, to)
}

/// The complementary filters share the same four independent index totals.
pub fn selected(id: AgeAggregateId, totals: &PriceTotals<4>) -> (i64, i128) {
    let (positive, negative) = match cut_of(id) {
        None => (0, None),
        Some((cutoff, true)) => (CUTOFF_FILTERS[cutoff], None),
        Some((cutoff, false)) => (0, Some(CUTOFF_FILTERS[cutoff])),
    };
    (
        totals.sats[positive] - negative.map_or(0, |i| totals.sats[i]),
        totals.cap[positive] - negative.map_or(0, |i| totals.cap[i]),
    )
}
