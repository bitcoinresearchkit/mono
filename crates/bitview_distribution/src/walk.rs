//! The one walk of the UTXO set's history. Everything that follows the coins block by block (the
//! age cohorts, the price index, the URPD cells) is a [`Consumer`] of it: one state of the
//! origins, one price and time per block, one scan of the age boundaries and one read of each
//! block's changes, whoever reads them.

use std::mem;

use bitview_cohort::{AGE_RANGE_COUNT, Age, AgeRangeId, for_each_age_crossing};
use bitview_compute::{Inputs, WRITE_INTERVAL};
use brk_error::{Error, Result};
use brk_types::{Cents, Height, Timestamp, Version};
use statedb::{Amount, BlockDiff, Reader, State};
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

/// The origins before a consumer's first block, which it restores its state from. Prices and
/// times cover the whole walk, by height.
pub struct Origins<'a> {
    /// What each origin holds.
    pub amounts: &'a [Amount],
    /// The hash of the last block applied, zero before the first.
    pub hash: [u8; 32],
    pub prices: &'a [Cents],
    pub timestamps: &'a [Timestamp],
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
    /// The origins that aged past a boundary since the block before, by boundary then height.
    pub crossings: &'a [Crossing],
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

    /// The block's effect on the sats held at each age, in the order every reader applies it:
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

    /// Saves what it computed for the blocks before `end`.
    fn save(&mut self, end: usize) -> Result<()>;
}

/// The walk's own state, kept between updates.
#[derive(Default)]
pub struct Walk {
    origins: State,
    prices: Vec<Cents>,
    timestamps: Vec<Timestamp>,
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

    /// Walks from the lowest block a consumer has yet to compute through the sources' end. Each
    /// consumer joins at its own start, from the state it kept if the last walk left it there on
    /// this chain, restored from the origins otherwise; all save every [`WRITE_INTERVAL`] blocks
    /// and at the end.
    pub fn run(&mut self, sources: Sources<'_>, consumers: &mut [&mut dyn Consumer]) -> Result<()> {
        // Where the last walk left its consumers' states, if it completed.
        let kept =
            mem::take(&mut self.complete).then(|| (self.origins.len(), self.origins.hash()));
        let Inputs { version, len: end } = sources.inputs(Version::ZERO);
        let Some(start) = consumers.iter().map(|consumer| consumer.start()).min() else {
            return Ok(());
        };
        if kept.is_none()
            || self.version != version
            || !sources.history.resumes(&self.origins, start)?
        {
            self.origins = sources.history.state_at(start)?;
            self.prices.clear();
            self.timestamps.clear();
            self.positions = [0; AGE_RANGE_COUNT - 1];
            self.version = version;
        }
        let Self {
            origins,
            prices,
            timestamps,
            positions,
            crossings,
            ..
        } = self;
        sources.prices.read_into_at(prices.len(), end, prices);
        sources
            .timestamps
            .read_into_at(timestamps.len(), end, timestamps);
        if prices.len() != end || timestamps.len() != end {
            return Err(Error::Internal("incomplete walk price or timestamp history"));
        }
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
                crossings,
                diff,
                amounts: state.amounts(),
            };
            let saves = (height + 1).is_multiple_of(WRITE_INTERVAL) || height + 1 == end;
            for consumer in consumers.iter_mut().filter(|c| c.start() <= height) {
                consumer.apply(&block)?;
                if saves {
                    consumer.save(height + 1)?;
                }
            }
        }
        self.complete = true;
        Ok(())
    }
}
