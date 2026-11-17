use std::{array, io, ops::Range, path::Path};

use bitview_cohort::{AGE_RANGE_COUNT, AgeRange, AgeRangeId};
use bitview_primitives::CentsCompact;
use brk_error::{Error, Result};
use brk_types::{Cents, Height, Version};
use statedb::CellHistory;

use crate::{
    COST_BASIS_PRICE_DIGITS, ProjectedBucket,
    price_place::{PRICE_PLACES, place_price, price_place},
    projection::Projection,
};

/// Revision of the cells' layout and meaning, part of their version.
const FORMAT: Version = Version::ONE;

/// The UTXO set's realized price distribution after every block: an age range's sats at one
/// rounded creation price is a cell (key `age × PRICE_PLACES + place`), kept as each block's
/// changes with periodic checkpoints. A read restores one block's cells without the origins.
pub struct UrpdHistory {
    cells: CellHistory,
    live: Option<Live>,
    /// The block being pushed's cell changes.
    changes: Vec<(u32, i64)>,
}

/// Where the producer stands after its blocks, kept between updates.
struct Live {
    urpd: UrpdCells,
    /// Each origin's price place.
    places: Vec<u32>,
}

impl UrpdHistory {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            cells: CellHistory::open(path)?,
            live: None,
            changes: Vec::new(),
        })
    }

    pub fn open_reader(path: &Path) -> Result<Self> {
        Ok(Self {
            cells: CellHistory::open_reader(path)?,
            live: None,
            changes: Vec::new(),
        })
    }

    /// The blocks recorded.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// The summed versions of what the cells mean and are made from.
    pub fn version(&self) -> Version {
        Version::from(self.cells.version() as u32)
    }

    /// The cells after `end` blocks, reading nothing past `ceiling` blocks.
    pub fn at(&self, end: usize, ceiling: usize) -> Result<UrpdCells> {
        let mut urpd = UrpdCells::default();
        urpd.hash = self.cells.restore(end, ceiling, |key, change| urpd.add(key, change))?;
        Ok(urpd)
    }

    /// Whether `cells`, after `end` blocks, follow the recorded chain.
    pub(crate) fn follows(&self, cells: &UrpdCells, end: usize) -> Result<bool> {
        Ok(end <= self.len() && self.cells.hash(end)? == cells.hash)
    }

    /// Moves `urpd`, the cells after `range.start` blocks, through each block of `range`,
    /// handing `each` the block's height and the cells after it.
    pub(crate) fn walk(
        &self,
        range: Range<usize>,
        urpd: &mut UrpdCells,
        mut each: impl FnMut(Height, &UrpdCells) -> Result<()>,
    ) -> Result<()> {
        let mut height = range.start;
        self.cells.blocks(range, |hash, changes| {
            for &(key, change) in changes {
                urpd.add(key, change)?;
            }
            urpd.hash = hash;
            each(Height::from(height), urpd)?;
            height += 1;
            Ok(())
        })
    }

    /// Readies the record to take blocks from `from` at most, made from sources of these
    /// versions: a changed version starts the cells over. Returns the block it resumes at.
    /// Under the exit lock, as [`Self::restore`] and [`Self::commit`]: each can publish.
    pub fn begin(
        &mut self,
        from: usize,
        prices: Version,
        timestamps: Version,
        (spends, creations): (u64, u64),
    ) -> Result<usize> {
        let version = u64::from(u32::from(
            FORMAT
                + prices
                + timestamps
                + Version::from(spends as u32)
                + Version::from(creations as u32),
        ));
        // A changed version starts the cells over, and the live cells then resume nothing.
        if self.cells.version() != version {
            self.live = None;
        }
        self.cells.validate_version(version)?;
        Ok(from.min(self.cells.len()))
    }

    /// Whether the producer still stands where its last recorded block left it.
    pub fn kept(&self) -> bool {
        self.live.is_some()
    }

    /// The producer's position after `start` blocks (the recorded ones past it dropped), the
    /// last one's hash `hash`: the cells from their own record, each block's place from `prices`.
    pub fn restore(&mut self, start: usize, hash: [u8; 32], prices: &[Cents]) -> Result<()> {
        self.live = None;
        self.cells.truncate(start)?;
        let urpd = self.at(start, start)?;
        if urpd.hash != hash {
            return Err(Error::Internal("URPD history does not match the UTXO set history"));
        }
        self.live = Some(Live {
            urpd,
            places: round(prices)?,
        });
        Ok(())
    }

    /// Records the next block, its hash `hash`, from `changes`: (age, origin, signed sats) in
    /// any order. `prices` covers every origin a change names.
    pub fn push(
        &mut self,
        hash: [u8; 32],
        prices: &[Cents],
        changes: impl IntoIterator<Item = (AgeRangeId, usize, i64)>,
    ) -> Result<()> {
        let Live { urpd, places } = (self.live.as_mut())
            .ok_or(Error::Internal("URPD history pushed to before its restore"))?;
        if places.len() < prices.len() {
            places.extend(round(&prices[places.len()..])?);
        }
        let cells = &mut self.changes;
        cells.clear();
        cells.extend(changes.into_iter().map(|(age, origin, sats)| {
            ((age.index() * PRICE_PLACES) as u32 + places[origin], sats)
        }));
        // One change per cell, by key, none zero.
        cells.sort_unstable_by_key(|&(key, _)| key);
        cells.dedup_by(|(key, sats), (kept, total)| {
            let same = key == kept;
            if same {
                *total += *sats;
            }
            same
        });
        cells.retain(|&(_, sats)| sats != 0);
        for &(key, sats) in cells.iter() {
            urpd.add(key, sats)?;
        }
        urpd.hash = hash;
        self.cells.push(urpd.hash, cells, || urpd.nonzero())?;
        Ok(())
    }

    /// Commits the blocks pushed since the last commit.
    pub fn commit(&mut self) -> Result<()> {
        Ok(self.cells.commit()?)
    }
}

/// Each price's place once rounded.
fn round(prices: &[Cents]) -> Result<Vec<u32>> {
    prices
        .iter()
        .map(|&price| {
            if price.is_nan() {
                return Err(Error::Internal("unpriced block in the URPD history"));
            }
            let rounded = CentsCompact::from(price).round_to_significant(COST_BASIS_PRICE_DIGITS);
            Ok(price_place(rounded) as u32)
        })
        .collect()
}

/// One block's distribution: each age range's sats over every price place, with each place's
/// nonzero ages, so a projection walks the occupied places in price order.
pub struct UrpdCells {
    amounts: [Vec<u64>; AGE_RANGE_COUNT],
    occupied: Vec<u32>,
    /// Past the highest place ever occupied: prices above it have never held sats, so a
    /// projection stops there (about 37,000 of the 64,000 places at today's prices).
    top: usize,
    hash: [u8; 32],
}

impl Default for UrpdCells {
    fn default() -> Self {
        Self {
            amounts: array::from_fn(|_| vec![0; PRICE_PLACES]),
            occupied: vec![0; PRICE_PLACES],
            top: 0,
            hash: [0; 32],
        }
    }
}

impl UrpdCells {
    /// The hash of the block the cells follow.
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    fn add(&mut self, key: u32, change: i64) -> io::Result<()> {
        let (age, place) = (key as usize / PRICE_PLACES, key as usize % PRICE_PLACES);
        let column = (self.amounts.get_mut(age))
            .ok_or_else(|| io::Error::other("URPD cell outside the age ranges"))?;
        let sats = &mut column[place];
        *sats = (sats.checked_add_signed(change))
            .ok_or_else(|| io::Error::other("URPD cell out of range"))?;
        if *sats == 0 {
            self.occupied[place] &= !(1 << age);
        } else {
            self.occupied[place] |= 1 << age;
            self.top = self.top.max(place + 1);
        }
        Ok(())
    }

    /// Every nonzero cell by ascending key.
    fn nonzero(&self) -> impl Iterator<Item = (u32, u64)> + '_ {
        self.amounts.iter().enumerate().flat_map(|(age, column)| {
            column
                .iter()
                .enumerate()
                .filter(|&(_, &sats)| sats != 0)
                .map(move |(place, &sats)| ((age * PRICE_PLACES + place) as u32, sats))
        })
    }

    /// Projects all requested weights together, visiting each occupied price's ages once, in
    /// price order. Weighted entries keep the weights' order, a missing weight gives zero, and
    /// rounding happens after combining every age at a price.
    pub fn project<'a, const N: usize, const C: usize>(
        &'a self,
        weights: &[Option<&AgeRange<f64>>; N],
        cohorts: [&[AgeRangeId]; C],
    ) -> impl Iterator<Item = ProjectedBucket<N, C>> + 'a + use<'a, N, C> {
        let projection = Projection::new(weights, cohorts);
        (self.occupied[..self.top].iter().enumerate())
            .filter(|&(_, &ages)| ages != 0)
            .filter_map(move |(place, &ages)| {
                projection.bucket(place_price(place), &self.amounts, place, ages)
            })
    }
}
