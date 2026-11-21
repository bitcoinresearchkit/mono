//! The one walk of the UTXO set's history. Everything that follows the coins block by block (the
//! age cohorts, the price index, the URPD cells) is a [`Consumer`] of it: one state of the
//! origins, one price and time per block, one order of the origins by price, one scan of the age
//! boundaries and one read of each block's changes, whoever reads them.

use std::mem;

use bitview_cohort::{
    AGE_RANGE_COUNT, Age, AgeRangeId, age_boundary_positions, for_each_age_crossing,
};
use bitview_compute::{Inputs, WRITE_INTERVAL};
use bitview_primitives::CentsCompact;
use brk_error::{Error, Result};
use brk_types::{Cents, Height, Timestamp, Version};
pub use statedb::Reader;
use statedb::{Amount, BlockDiff, State};
use vecdb::{AnyVec, ReadableBoxedVec, ReadableVec};

/// What the walk reads: the history, and each block's price and monotonic time.
#[derive(Clone, Copy)]
pub struct Sources<'a> {
    pub history: &'a Reader<'a>,
    pub prices: &'a ReadableBoxedVec<Height, Cents>,
    pub timestamps: &'a ReadableBoxedVec<Height, Timestamp>,
}

impl Sources<'_> {
    /// What a computation over the walk reads, under its own `version`: the sources' versions and
    /// the blocks all three cover.
    pub fn inputs(&self, version: Version) -> Inputs {
        let mut inputs = Inputs::new(
            version + history_version(self.history),
            [self.prices as &dyn AnyVec, self.timestamps],
        );
        inputs.len = inputs.len.min(self.history.len());
        inputs
    }
}

/// The history's spends and creations versions as one `Version`, folding in each u64's high half
/// instead of truncating it.
fn history_version(history: &Reader<'_>) -> Version {
    let (spends, creations) = history.versions();
    let fold = |version: u64| Version::new(version as u32) + Version::new((version >> 32) as u32);
    fold(spends) + fold(creations)
}

/// The walk's origins in order of price. A coin's price is its origin's, fixed for good, so this
/// one order answers for every cohort which coins the spot passes when it moves: no cohort keeps
/// its coins by price.
#[derive(Default)]
pub struct PriceOrder(Vec<(CentsCompact, u32)>);

impl PriceOrder {
    /// Takes in the origins it lacks: `prices` is every origin's, by height.
    fn extend(&mut self, prices: &[Cents]) {
        let known = self.0.len();
        self.0.extend(
            (prices.iter().enumerate().skip(known))
                .map(|(origin, &price)| (CentsCompact::from(price), origin as u32)),
        );
        let Some(&lowest) = self.0[known..].iter().min() else {
            return;
        };
        // The origins under the lowest new one keep their place: a block at the tip sorts with
        // the ones priced over it (some 90,000 at today's prices), not with all of them. `sort`
        // because it merges runs already in order, which `sort_unstable` sorts again (6 ms
        // against 116 ms for one new origin among all 976,000).
        let settled = self.0[..known].partition_point(|&held| held < lowest);
        self.0[settled..].sort();
    }

    /// The origins priced over the lower of two spots and up to the higher, with their price:
    /// the ones whose coins change side of the spot when it goes from one to the other. Every
    /// origin of the walk is in the order, the ones past a reader's own block too.
    pub fn passed(
        &self,
        from: CentsCompact,
        to: CentsCompact,
    ) -> impl Iterator<Item = (CentsCompact, usize)> + '_ {
        let through = |spot| self.0.partition_point(|&(price, _)| price <= spot);
        (self.0[through(from.min(to))..through(from.max(to))].iter())
            .map(|&(price, origin)| (price, origin as usize))
    }
}

/// The origins before a consumer's first block, which it restores its state from. Prices and
/// times cover the whole walk, by height.
pub struct Origins<'a> {
    /// What each origin holds.
    pub amounts: &'a [Amount],
    /// The hash of the last block applied, zero before the first.
    pub hash: [u8; 32],
    pub prices: &'a [Cents],
    pub timestamps: &'a [Timestamp],
    /// The first origin younger than each boundary, at the last block applied.
    pub age_positions: &'a [usize; AGE_RANGE_COUNT - 1],
}

/// An origin whose age passed a boundary since the block before.
pub struct Crossing {
    pub origin: usize,
    pub younger: AgeRangeId,
    pub older: AgeRangeId,
    /// When it crossed: its own time plus the boundary.
    pub at: u32,
    /// What it held then: the block's spends come after.
    pub amount: Amount,
}

/// One block of the walk. Prices and times cover the whole walk, by height.
pub struct Block<'a> {
    pub height: usize,
    /// The block's hash in the history's chain.
    pub hash: [u8; 32],
    pub prices: &'a [Cents],
    pub timestamps: &'a [Timestamp],
    /// Every origin of the walk by price, the ones past this block too.
    pub order: &'a PriceOrder,
    /// The origins that aged past a boundary since the block before, by boundary then height.
    pub crossings: &'a [Crossing],
    /// The first origin younger than each boundary, after this block.
    pub age_positions: &'a [usize; AGE_RANGE_COUNT - 1],
    /// What the block created and removed.
    pub diff: BlockDiff<'a>,
    /// What each origin holds after the block.
    pub amounts: &'a [Amount],
}

impl Block<'_> {
    pub fn price(&self) -> Cents {
        self.prices[self.height]
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamps[self.height]
    }

    /// The block's effect on the sats held at each age, in the order URPD applies it:
    /// the origins that aged, as they stood before the block, move to the older range; the
    /// block's new outputs arrive under an hour old; what it removed leaves the age its origin
    /// has at the block.
    pub fn moves(&self) -> impl Iterator<Item = Move> + '_ {
        let current = self.timestamp();
        let aged = self.crossings.iter().map(|crossing| Move {
            origin: crossing.origin,
            sats: crossing.amount.sats,
            from: Some(crossing.younger),
            to: Some(crossing.older),
        });
        let created = Move {
            origin: self.height,
            sats: self.diff.created.sats,
            from: None,
            to: Some(AgeRangeId::Under1H),
        };
        let removed = self.diff.removed().map(move |(origin, amount)| {
            let origin = origin as usize;
            Move {
                origin,
                sats: amount.sats,
                from: Some(AgeRangeId::from(Age::new(current, self.timestamps[origin]))),
                to: None,
            }
        });
        aged.chain([created])
            .chain(removed)
            .filter(|moved| moved.sats != 0)
    }
}

/// Sats of one origin moving between ages, never zero: from none when created, to none when
/// removed.
pub struct Move {
    pub origin: usize,
    pub sats: u64,
    pub from: Option<AgeRangeId>,
    pub to: Option<AgeRangeId>,
}

/// A computation fed by the walk: its outputs end at its `start`, and it follows every block from
/// there.
pub trait Consumer {
    /// The first block it has yet to compute, fixed for the walk.
    fn start(&self) -> usize;

    /// Whether it still holds the state it kept when the last walk completed (its owner keeps
    /// it only then).
    fn kept(&self) -> bool;

    /// Rebuilds its state from the origins before its first block.
    fn restore(&mut self, origins: &Origins<'_>) -> Result<()>;

    fn apply(&mut self, block: &Block<'_>) -> Result<()>;

    /// Saves what it computed for the blocks applied.
    fn save(&mut self) -> Result<()>;
}

/// The walk's own state, kept between updates.
#[derive(Default)]
pub struct Walk {
    origins: State,
    prices: Vec<Cents>,
    timestamps: Vec<Timestamp>,
    /// The origins of `prices` by price.
    order: PriceOrder,
    /// The age boundaries' scan positions, valid for `origins`.
    positions: [usize; AGE_RANGE_COUNT - 1],
    /// The block being applied's crossings.
    crossings: Vec<Crossing>,
    /// The sources' version the state was made under.
    version: Version,
    /// Whether the last walk completed: only then do the state, and what its consumers kept,
    /// stand where it ended.
    complete: bool,
}

impl Walk {
    /// Every block's price through the last walk's end.
    pub fn prices(&self) -> &[Cents] {
        &self.prices
    }

    /// Every block's monotonic time through the last walk's end.
    pub fn timestamps(&self) -> &[Timestamp] {
        &self.timestamps
    }

    /// Every block through the last walk's end by price.
    pub fn order(&self) -> &PriceOrder {
        &self.order
    }

    /// Walks from the lowest block a consumer has yet to compute through the sources' end. Each
    /// consumer joins at its own start, from the state it kept if the last walk left it there on
    /// this chain, restored from the origins otherwise; all save every [`WRITE_INTERVAL`] blocks
    /// and at the end. `dyn`: the consumers are of different types in one list, and a call each
    /// per block is nothing beside the block's work.
    pub fn run(&mut self, sources: Sources<'_>, consumers: &mut [&mut dyn Consumer]) -> Result<()> {
        // Where the last walk left its consumers' states, if it completed.
        let kept =
            mem::take(&mut self.complete).then(|| (self.origins.len(), self.origins.hash()));
        let Inputs { version, len: end } = sources.inputs(Version::ZERO);
        let Some(start) = consumers.iter().map(|consumer| consumer.start()).min() else {
            return Ok(());
        };
        let restore = kept.is_none()
            || self.version != version
            || !sources.history.resumes(&self.origins, start)?;
        if restore {
            self.origins = sources.history.state_at(start)?;
            self.prices.clear();
            self.timestamps.clear();
            self.order = PriceOrder::default();
            self.version = version;
        }
        let Self {
            origins,
            prices,
            timestamps,
            order,
            positions,
            crossings,
            ..
        } = self;
        crate::read_basis_prices(sources.prices, end, prices);
        sources
            .timestamps
            .read_into_at(timestamps.len(), end, timestamps);
        if prices.len() != end || timestamps.len() != end {
            return Err(Error::Internal("incomplete walk price or timestamp history"));
        }
        if restore {
            *positions = age_boundary_positions(&timestamps[..start]);
        }
        order.extend(prices);
        let mut cursor = sources.history.cursor(origins)?;
        for height in start..=end {
            let state = cursor.state();
            for consumer in consumers.iter_mut().filter(|c| c.start() == height) {
                if !(consumer.kept() && kept == Some((height, state.hash()))) {
                    consumer.restore(&Origins {
                        amounts: state.amounts(),
                        hash: state.hash(),
                        prices,
                        timestamps,
                        age_positions: positions,
                    })?;
                }
            }
            if height == end {
                break;
            }
            crossings.clear();
            for_each_age_crossing(
                &timestamps[..height],
                timestamps[height],
                positions,
                |origin, younger, older, at| {
                    crossings.push(Crossing {
                        origin,
                        younger,
                        older,
                        at,
                        amount: state.amounts()[origin],
                    });
                },
            );
            let (diff, state) = cursor
                .advance()?
                .ok_or(Error::Internal("history short of the walk"))?;
            let block = Block {
                height,
                hash: state.hash(),
                prices,
                timestamps,
                order,
                crossings,
                age_positions: positions,
                diff,
                amounts: state.amounts(),
            };
            let saves = (height + 1).is_multiple_of(WRITE_INTERVAL) || height + 1 == end;
            for consumer in consumers.iter_mut().filter(|c| c.start() <= height) {
                consumer.apply(&block)?;
                if saves {
                    consumer.save()?;
                }
            }
        }
        self.complete = true;
        Ok(())
    }
}
