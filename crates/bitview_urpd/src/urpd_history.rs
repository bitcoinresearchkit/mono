use std::{io, ops::Range, path::Path};

use bitview_cohort::{AGE_RANGE_COUNT, AgeRange, AgeRangeId};
use bitview_types::UrpdAggregation;
use brk_error::{Error, Result};
use brk_types::{Cents, Height, Version};
use statedb::CellHistory;

use crate::{ProjectedBucket, price_place::{GRID, PLACE_BITS, PLACE_MASK}, projection::Projection};

/// Revision of the cells' layout and meaning, part of their version.
const FORMAT: Version = Version::new(3);

/// The UTXO set's realized price distribution after every block: an age range's sats at one
/// creation-price interval is a cell, ordered by age and then descending price, kept as each block's
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
            places: price_places(prices)?,
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
            places.extend(price_places(&prices[places.len()..])?);
        }
        let cells = &mut self.changes;
        cells.clear();
        cells.extend(changes.into_iter().map(|(age, origin, sats)| {
            (cell_key(age.index(), places[origin]), sats)
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

// A checkpoint's first key for an age names that column's highest price. Restoring
// it can allocate the entire column before writing its balances.
fn cell_key(age: usize, place: u32) -> u32 {
    ((age as u32) << PLACE_BITS) | (PLACE_MASK ^ place)
}

/// Locate each exact origin price before any grouping or valuation.
fn price_places(prices: &[Cents]) -> Result<Vec<u32>> {
    prices
        .iter()
        .map(|&price| {
            if price.is_nan() {
                return Err(Error::Internal("unpriced block in the URPD history"));
            }
            let cents = price.inner();
            if cents >= u64::from(u32::MAX) {
                return Err(Error::Internal(
                    "URPD creation price exceeds the compact price range",
                ));
            }
            Ok(GRID.place(cents as u32))
        })
        .collect()
}

/// One block's distribution: each age range's sats over every price place, with each place's
/// nonzero ages, so a projection walks the occupied places in price order.
#[derive(Default)]
pub struct UrpdCells {
    /// Only the prefix an age has reached. Most of the full grid is above historical prices.
    amounts: [Vec<u64>; AGE_RANGE_COUNT],
    occupied: Vec<u32>,
    hash: [u8; 32],
}

// The first allocation is zero-filled by the allocator. Most checkpoint columns
// reach their final length on their first entry.
#[cold]
fn grow<T: Clone + Default>(column: &mut Vec<T>, len: usize) {
    if column.is_empty() {
        *column = vec![T::default(); len];
    } else {
        column.resize(len, T::default());
    }
}

impl UrpdCells {
    /// The hash of the block the cells follow.
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    fn add(&mut self, key: u32, change: i64) -> io::Result<()> {
        let age = (key >> PLACE_BITS) as usize;
        let place = (!key & PLACE_MASK) as usize;
        let column = (self.amounts.get_mut(age))
            .ok_or_else(|| io::Error::other("URPD cell outside the age ranges"))?;
        if place >= column.len() {
            if place >= GRID.len() {
                return Err(io::Error::other("URPD cell outside the price grid"));
            }
            grow(column, place + 1);
            if place >= self.occupied.len() {
                grow(&mut self.occupied, place + 1);
            }
        }
        let sats = &mut column[place];
        *sats = (sats.checked_add_signed(change))
            .ok_or_else(|| io::Error::other("URPD cell out of range"))?;
        if *sats == 0 {
            self.occupied[place] &= !(1 << age);
        } else {
            self.occupied[place] |= 1 << age;
        }
        Ok(())
    }

    /// Every nonzero cell by ascending key.
    fn nonzero(&self) -> impl Iterator<Item = (u32, u64)> + '_ {
        self.amounts.iter().enumerate().flat_map(|(age, column)| {
            column
                .iter()
                .enumerate()
                .rev()
                .filter(|&(_, &sats)| sats != 0)
                .map(move |(place, &sats)| (cell_key(age, place as u32), sats))
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
        self.project_buckets(UrpdAggregation::Raw, weights, cohorts)
    }

    /// Keep the raw price for valuation. Within one raw price split across requested
    /// buckets, differences of cumulative whole sats allocate its rounding remainder in
    /// price order and give the same weighted total as the raw projection.
    pub fn project_buckets<'a, const N: usize, const C: usize>(
        &'a self,
        aggregation: UrpdAggregation,
        weights: &[Option<&AgeRange<f64>>; N],
        cohorts: [&[AgeRangeId]; C],
    ) -> impl Iterator<Item = ProjectedBucket<N, C>> + 'a + use<'a, N, C> {
        let projection = Projection::new(weights, cohorts);
        let rows = GRID.rows(aggregation);
        let count = rows.partition_point(|row| (row.start as usize) < self.occupied.len());
        let mut previous = None;
        rows[..count].iter().filter_map(move |row| {
            let start = if N == 0 { row.start } else { row.raw_start } as usize;
            let end = (row.end as usize).min(self.occupied.len());
            let mut bucket = if start + 1 == end {
                projection.bucket(row, self.occupied[start], |age| self.amounts[age][start])
            } else {
                let occupied = self.occupied[start..end].iter().fold(0, |mask, ages| mask | ages);
                projection.bucket(row, occupied, |age| {
                    let column = &self.amounts[age];
                    column[start..end.min(column.len())].iter().sum()
                })
            }?;
            if N != 0
                && aggregation != UrpdAggregation::Raw
                && let Some(prior) = previous.replace(bucket)
                && prior.price == bucket.price
            {
                for (value, prior) in bucket.raw.iter_mut().zip(prior.raw) {
                    *value -= prior;
                }
                for (values, prior) in bucket.weighted.iter_mut().zip(prior.weighted) {
                    for (value, prior) in values.iter_mut().zip(prior) {
                        *value -= prior;
                    }
                }
            }
            Some(bucket)
        })
    }
}
