use bitview_primitives::CentsCompact;
use brk_types::Cents;
use statedb::Amount;

use super::BucketedPrice;

/// What the age index keeps of the origins holding coins, beside its price index, told of each as the history moves:
/// their extremes ([`HeldExtremes`]), or nothing (`()`, for an owner that reads none of it).
pub trait HeldOrigins: Default {
    /// Room for `len` origins, those held kept.
    fn reserve(&mut self, len: usize);
    /// Every origin of `amounts` holding coins, at its price.
    fn restore(&mut self, amounts: &[Amount], prices: &[BucketedPrice]);
    /// An origin the block being applied spends from.
    fn spend_from(&mut self, origin: u32);
    /// The block at `height` applied (`amounts`, as it left them): its own origin held if it kept any, each it spent
    /// from let go if it emptied it.
    fn settle(&mut self, height: usize, amounts: &[Amount], prices: &[BucketedPrice]);
}

impl HeldOrigins for () {
    fn reserve(&mut self, _: usize) {}
    fn restore(&mut self, _: &[Amount], _: &[BucketedPrice]) {}
    fn spend_from(&mut self, _: u32) {}
    fn settle(&mut self, _: usize, _: &[Amount], _: &[BucketedPrice]) {}
}

/// The lowest and highest price among the origin heights still holding coins, over any run of
/// heights (prices over `CentsCompact`'s ceiling, $42,949,672.94, held there, as the index's are). A coin's cost basis is its origin block's price and an age cohort is one run of
/// origin heights (its youngest or its oldest), so a run's extremes are a cohort's, exactly. An
/// origin empties once at most (coins are created only at their own height): its updates over
/// all history are as many as the heights.
#[derive(Default)]
pub struct HeldExtremes {
    tree: MinMaxTree,
    /// The origins the block being applied spent from, let go once it's applied if it emptied
    /// them.
    spent: Vec<u32>,
}

impl HeldOrigins for HeldExtremes {
    fn reserve(&mut self, len: usize) {
        self.tree.reserve(len);
    }

    fn restore(&mut self, amounts: &[Amount], prices: &[BucketedPrice]) {
        self.tree.reset();
        for (h, amount) in amounts.iter().enumerate() {
            if amount.sats != 0 {
                self.tree.set_leaf(h, u32::from(prices[h].price()));
            }
        }
        self.tree.build();
    }

    fn spend_from(&mut self, origin: u32) {
        self.spent.push(origin);
    }

    fn settle(&mut self, height: usize, amounts: &[Amount], prices: &[BucketedPrice]) {
        if amounts[height].sats != 0 {
            self.tree.set(height, u32::from(prices[height].price()));
        }
        for origin in self.spent.drain(..) {
            let origin = origin as usize;
            if amounts[origin].sats == 0 && self.tree.holds(origin) {
                self.tree.clear(origin);
            }
        }
    }
}

impl HeldExtremes {
    /// The lowest and highest price of the origins held in `from..to`; NaN both where none is.
    pub fn range(&self, from: usize, to: usize) -> (Cents, Cents) {
        let cents = |value: u32| Cents::from(CentsCompact::from(value));
        match self.tree.range(from, to) {
            Some((low, high)) => (cents(low), cents(high)),
            None => (Cents::NAN, Cents::NAN),
        }
    }
}

/// The lowest and highest value over any run of positions, each position a value or none: a
/// segment tree, its leaves from `size`, each node over its two children.
#[derive(Default)]
struct MinMaxTree {
    /// A node's lowest (`u32::MAX` where none) and highest (`0` where none: the lowest tells), as `CentsCompact`'s.
    low: Vec<u32>,
    high: Vec<u32>,
    size: usize,
}

impl MinMaxTree {
    /// None held: the lowest's past any price (`CentsCompact`'s NaN, never an origin's), the highest's under any.
    const NONE: (u32, u32) = (u32::MAX, 0);

    fn reserve(&mut self, len: usize) {
        if len <= self.size {
            return;
        }
        let size = len.next_power_of_two();
        let mut grown = Self {
            low: vec![Self::NONE.0; 2 * size],
            high: vec![Self::NONE.1; 2 * size],
            size,
        };
        grown.low[size..size + self.size].copy_from_slice(&self.low[self.size..]);
        grown.high[size..size + self.size].copy_from_slice(&self.high[self.size..]);
        grown.build();
        *self = grown;
    }

    fn reset(&mut self) {
        self.low.fill(Self::NONE.0);
        self.high.fill(Self::NONE.1);
    }

    /// A leaf alone: [`Self::build`] after a batch.
    fn set_leaf(&mut self, at: usize, value: u32) {
        debug_assert!(at < self.size, "position {at} past the reserved room");
        debug_assert!(value != Self::NONE.0, "a value can't be the lowest's none");
        (self.low[self.size + at], self.high[self.size + at]) = (value, value);
    }

    fn build(&mut self) {
        for node in (1..self.size).rev() {
            self.pull(node);
        }
    }

    fn set(&mut self, at: usize, value: u32) {
        self.set_leaf(at, value);
        self.pull_up(at);
    }

    fn clear(&mut self, at: usize) {
        (self.low[self.size + at], self.high[self.size + at]) = Self::NONE;
        self.pull_up(at);
    }

    fn holds(&self, at: usize) -> bool {
        self.low[self.size + at] != Self::NONE.0
    }

    fn pull_up(&mut self, at: usize) {
        let mut node = (self.size + at) / 2;
        while node > 0 {
            self.pull(node);
            node /= 2;
        }
    }

    #[inline]
    fn pull(&mut self, node: usize) {
        self.low[node] = self.low[2 * node].min(self.low[2 * node + 1]);
        self.high[node] = self.high[2 * node].max(self.high[2 * node + 1]);
    }

    /// The lowest and highest over `from..to`, none where it holds none.
    fn range(&self, from: usize, to: usize) -> Option<(u32, u32)> {
        debug_assert!(from <= to && to <= self.size);
        let (mut low, mut high) = Self::NONE;
        let (mut l, mut r) = (from + self.size, to + self.size);
        while l < r {
            if l & 1 == 1 {
                (low, high) = (low.min(self.low[l]), high.max(self.high[l]));
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                (low, high) = (low.min(self.low[r]), high.max(self.high[r]));
            }
            (l, r) = (l / 2, r / 2);
        }
        (low != Self::NONE.0).then_some((low, high))
    }
}
