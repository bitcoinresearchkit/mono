use bitview_cohort::{AGE_RANGE_COUNT, AgeAggregateId, AgeRangeId};
use bitview_primitives::CentsCompact;
use brk_error::{Error, Result};
use brk_types::Cents;

use super::{BucketedPrice, HeldExtremes, PriceIndex, PriceTotals};
use crate::walk::{Block, Origins};

/// The sats and capital held at each price under four age filters (all, under four, five and
/// six months), and the extremes of the origins holding coins: the
/// state of a walk consumer, which restores it from the origins and applies each block to it.
#[derive(Default)]
pub struct AgeIndex {
    pub index: PriceIndex<4>,
    held: HeldExtremes,
    /// Each block's price with its bucket, mapped once.
    prices: Vec<BucketedPrice>,
}

impl AgeIndex {
    pub fn restore(&mut self, origins: &Origins<'_>) -> Result<()> {
        self.prices.clear();
        self.bucket(origins.prices)?;
        self.index.reset();
        let amounts = origins.amounts;
        let cuts = cuts(origins.age_positions);
        for (h, amount) in amounts.iter().enumerate() {
            if amount.sats != 0 {
                self.index
                    .add_raw(self.prices[h], amount.sats as i64, filters(h, cuts));
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
        // Each crossing leaves just the filter for that boundary. New outputs enter every
        // filter and removals use the origins' ages after the crossings.
        for crossing in block.crossings {
            let [four, five, six] = CUTOFFS.map(|older| crossing.older == older);
            if four || five || six {
                index.add(
                    prices[crossing.origin],
                    -(crossing.amount.sats as i64),
                    [false, four, five, six],
                );
            }
        }
        let height = block.height;
        index.add(prices[height], block.diff.created.sats as i64, [true; 4]);
        if block.amounts[height].sats != 0 {
            held.insert(height, prices[height]);
        }
        let cuts = cuts(block.age_positions);
        for (origin, amount) in block.diff.removed() {
            let origin = origin as usize;
            index.add(prices[origin], -(amount.sats as i64), filters(origin, cuts));
            if block.amounts[origin].sats == 0 {
                held.remove(origin);
            }
        }
        Ok(())
    }

    /// The lowest and highest cost basis of a cohort's held origins at this block.
    pub fn extremes(&self, id: AgeAggregateId, block: &Block<'_>) -> (Cents, Cents) {
        let end = block.height + 1;
        let cuts = cuts(block.age_positions);
        let (from, to) = match cut_of(id) {
            Some((filter, true)) => (cuts[filter - 1], end),
            Some((filter, false)) => (0, cuts[filter - 1]),
            None => (0, end),
        };
        self.held.range(from, to)
    }

    /// Maps the prices not yet mapped.
    fn bucket(&mut self, prices: &[Cents]) -> Result<()> {
        let new = &prices[self.prices.len()..];
        if new.is_empty() {
            return Ok(());
        }
        if new.iter().any(|price| price.is_nan()) {
            return Err(Error::NotFound("invalid age-index price history".into()));
        }
        self.prices
            .extend(new.iter().map(|&price| BucketedPrice::from(CentsCompact::from(price))));
        self.held.reserve(self.prices.len());
        Ok(())
    }
}

#[inline]
fn cuts(positions: &[usize; AGE_RANGE_COUNT - 1]) -> [usize; 3] {
    CUTOFFS.map(|older| positions[older.index() - 1])
}

const CUTOFFS: [AgeRangeId; 3] = [
    AgeRangeId::From4MTo5M,
    AgeRangeId::From5MTo6M,
    AgeRangeId::From6MTo9M,
];

#[inline]
fn filters(origin: usize, [four, five, six]: [usize; 3]) -> [bool; 4] {
    [true, origin >= four, origin >= five, origin >= six]
}

/// A cohort as an age cut: its index filter and whether it's the young side.
fn cut_of(id: AgeAggregateId) -> Option<(usize, bool)> {
    match id {
        AgeAggregateId::All => None,
        AgeAggregateId::Under4M => Some((1, true)),
        AgeAggregateId::Over4M => Some((1, false)),
        AgeAggregateId::Sth => Some((2, true)),
        AgeAggregateId::Lth => Some((2, false)),
        AgeAggregateId::Under6M => Some((3, true)),
        AgeAggregateId::Over6M => Some((3, false)),
    }
}

/// The complementary filters share the same four independent index totals.
pub fn selected(id: AgeAggregateId, totals: &PriceTotals<4>) -> (i64, i128) {
    let (positive, negative) = match cut_of(id) {
        None => (0, None),
        Some((filter, true)) => (filter, None),
        Some((filter, false)) => (0, Some(filter)),
    };
    (
        totals.sats[positive] - negative.map_or(0, |i| totals.sats[i]),
        totals.cap[positive] - negative.map_or(0, |i| totals.cap[i]),
    )
}
