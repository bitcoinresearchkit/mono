use std::ops::{AddAssign, Range, SubAssign};

use bitview_compute::{WRITE_INTERVAL, write_chunks, write_locked};
use bitview_traversable::Traversable;
use bitview_urpd::ReplayInputs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{BLOCKS_PER_HALVING, Cents, CentsSats, Sats};
use rayon::{current_num_threads, prelude::*};
use statedb::Reader;
use vecdb::ReadableVec;

use super::{FloorBandId, FloorBands, ModeId, ModeVecs, Modes, Percentiles};

/// The first block whose outputs count toward a floor band: the second halving's, by which
/// Bedrock's calibration has left its early years behind.
pub const FLOOR_SUPPLY_START: usize = 2 * BLOCKS_PER_HALVING as usize;

// The pass's write chunks never straddle it.
const _: () = assert!(FLOOR_SUPPLY_START.is_multiple_of(WRITE_INTERVAL));

/// What a band holds: unspent supply and its capital (each output at its creation price).
#[derive(Clone, Copy, Default)]
pub struct Holding {
    pub supply: Sats,
    pub capital: CentsSats,
}

impl Holding {
    /// This holding after `change`, which never takes more than it holds.
    fn after(self, change: Change) -> Self {
        let (supply, capital) = (u64::from(self.supply), self.capital.inner());
        debug_assert!(
            supply.checked_add_signed(change.supply).is_some()
                && capital.checked_add_signed(change.capital).is_some()
        );
        Self {
            supply: Sats::from(supply.wrapping_add_signed(change.supply)),
            capital: CentsSats::new(capital.wrapping_add_signed(change.capital)),
        }
    }
}

/// A change in a band's holding.
#[derive(Clone, Copy, Default)]
struct Change {
    supply: i64,
    capital: i128,
}

impl Change {
    /// `sats` created at `price`.
    fn of(price: Cents, sats: u64) -> Self {
        Self {
            supply: sats as i64,
            capital: CentsSats::from_price_sats(price, Sats::from(sats)).inner() as i128,
        }
    }
}

impl AddAssign for Change {
    fn add_assign(&mut self, rhs: Self) {
        self.supply += rhs.supply;
        self.capital += rhs.capital;
    }
}

impl SubAssign for Change {
    fn sub_assign(&mut self, rhs: Self) {
        self.supply -= rhs.supply;
        self.capital -= rhs.capital;
    }
}

/// Each mode's holdings by the floor band spot sat in when they were created, from
/// [`FLOOR_SUPPLY_START`], less what has been spent since.
#[derive(Default)]
pub struct EntrySupply {
    /// Each block's spot price from [`FLOOR_SUPPLY_START`]: its outputs' creation price.
    prices: Vec<Cents>,
    /// Each block's band per mode from [`FLOOR_SUPPLY_START`]: where its outputs are held.
    bands: Vec<Modes<Option<FloorBandId>>>,
    /// The holdings before block `held_end`.
    held: Modes<FloorBands<Holding>>,
    held_end: usize,
}

impl EntrySupply {
    /// Prepares the pass over `range`: drops what a rewind invalidated, classifies each block up to
    /// its end from the stored floors and spot prices (all of them after a restart), and, unless
    /// the last pass ended at its start, restores the holdings there from the UTXO set.
    pub fn resume(
        &mut self,
        range: &Range<usize>,
        modes: &Modes<ModeVecs>,
        inputs: ReplayInputs<'_>,
    ) -> Result<()> {
        let index = |height: usize| height.max(FLOOR_SUPPLY_START) - FLOOR_SUPPLY_START;
        self.prices.truncate(index(range.start));
        self.bands.truncate(index(range.start));
        let classified = self.bands.len();
        let (from, end) = (
            FLOOR_SUPPLY_START + classified,
            range.end.max(FLOOR_SUPPLY_START),
        );
        self.prices
            .extend(inputs.prices.collect_range_dyn(from, end));
        self.bands.resize(index(end), Modes::default());
        // One mode's floors at a time bounds the memory a long rebuild takes.
        for &mode in ModeId::ALL {
            let stored = &modes.select(mode).prices_stored.floor;
            let floors = Percentiles::from_fn(|percentile| {
                stored.select(percentile).collect_range_at(from, end)
            });
            for (offset, (bands, &spot)) in self.bands[classified..]
                .iter_mut()
                .zip(&self.prices[classified..])
                .enumerate()
            {
                let block = Percentiles::from_fn(|percentile| floors.select(percentile)[offset]);
                *bands.select_mut(mode) = FloorBandId::locate(&block, spot);
            }
        }
        if self.held_end != range.start {
            self.held = Modes::default();
            if range.start > FLOOR_SUPPLY_START {
                let state = inputs.history.state_at(range.start)?;
                let amounts = &state.amounts()[FLOOR_SUPPLY_START..range.start];
                for ((amount, &price), bands) in amounts.iter().zip(&self.prices).zip(&self.bands) {
                    for held in held_in(&mut self.held, bands) {
                        *held = held.after(Change::of(price, amount.sats));
                    }
                }
            }
            self.held_end = range.start;
        }
        Ok(())
    }

    /// Pushes each mode's holdings after each block of the resumed `range`. Each chunk's changes
    /// come from its own read of the history, in parallel; carrying the holdings across is all
    /// that stays sequential.
    pub fn compute(
        &mut self,
        range: Range<usize>,
        history: &Reader<'_>,
        modes: &mut Modes<ModeVecs>,
        exit: &Exit,
    ) -> Result<()> {
        debug_assert_eq!(self.held_end, range.start);
        let chunks = write_chunks(range.clone()).collect::<Vec<_>>();
        for window in chunks.chunks(current_num_threads()) {
            let changes = window
                .par_iter()
                .map(|chunk| self.changes(chunk.clone(), history))
                .collect::<Result<Vec<_>>>()?;
            for changes in changes {
                let base = self.held;
                for change in &changes {
                    self.held = Modes::from_fn(|mode| {
                        let (base, change) = (base.select(mode), change.select(mode));
                        FloorBands::from_fn(|band| base.select(band).after(*change.select(band)))
                    });
                    for (vecs, held) in modes.iter_mut().zip(self.held.iter()) {
                        vecs.floor.push(held);
                    }
                }
            }
            write_locked(
                modes
                    .iter_mut()
                    .flat_map(|vecs| vecs.floor.iter_any_stored_mut()),
                exit,
            )?;
        }
        self.held_end = range.end;
        Ok(())
    }

    /// Each block's change in holdings since the start of `chunk`: its creations join its bands,
    /// its spends leave the bands of the blocks that created what they spend. None before
    /// [`FLOOR_SUPPLY_START`], which no chunk straddles.
    fn changes(
        &self,
        chunk: Range<usize>,
        history: &Reader<'_>,
    ) -> Result<Vec<Modes<FloorBands<Change>>>> {
        let mut changes = Vec::with_capacity(chunk.len());
        if chunk.start < FLOOR_SUPPLY_START {
            debug_assert!(chunk.end <= FLOOR_SUPPLY_START);
            changes.resize(chunk.len(), Modes::default());
            return Ok(changes);
        }
        let mut diffs = history.diffs(chunk.start)?;
        let mut change = Modes::<FloorBands<Change>>::default();
        for height in chunk {
            let diff = diffs.advance()?.expect("history covers the floors");
            let index = height - FLOOR_SUPPLY_START;
            for held in held_in(&mut change, &self.bands[index]) {
                *held += Change::of(self.prices[index], diff.created.sats);
            }
            for (origin, amount) in diff.removed() {
                if let Some(index) = (origin as usize).checked_sub(FLOOR_SUPPLY_START) {
                    for held in held_in(&mut change, &self.bands[index]) {
                        *held -= Change::of(self.prices[index], amount.sats);
                    }
                }
            }
            changes.push(change);
        }
        Ok(changes)
    }
}

/// Each mode's entry for its band of `bands`, for the modes that have one.
fn held_in<'a, T>(
    held: &'a mut Modes<FloorBands<T>>,
    bands: &'a Modes<Option<FloorBandId>>,
) -> impl Iterator<Item = &'a mut T> {
    held.iter_mut()
        .zip(bands.iter())
        .filter_map(|(held, band)| band.map(|band| held.select_mut(band)))
}
