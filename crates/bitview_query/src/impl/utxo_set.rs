use std::collections::BTreeMap;

use bitview_primitives::{Date, Day1};
use bitview_types::{UtxoChanges, UtxoOrigins, UtxoSet, UtxoSetDiff};
use brk_types::{Bitcoin, BlockHash, Height, Sats};
use vecdb::ReadableVec;

use crate::{Error, OptionData, Query, ReadPin, Result, check_bound};

fn btc(sats: u64) -> Bitcoin {
    Bitcoin::from(Sats::new(sats))
}

/// The published blocks a point covers: one block, or a UTC day's blocks.
pub struct UtxoSetPoint {
    first: Height,
    last: Height,
    hash: BlockHash,
    date: Date,
    settled: bool,
}

impl UtxoSetPoint {
    /// The last covered block's hash.
    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    /// Whether the URL pins the point below the reorg window: appends never change it.
    pub fn is_settled(&self) -> bool {
        self.settled
    }

    /// The point's exclusive end, once a reader of `history` blocks covers it.
    fn end_within(&self, history: usize) -> Result<usize> {
        let end = usize::from(self.last) + 1;
        if end > history {
            return Err(Error::StateUpdating);
        }
        Ok(end)
    }
}

impl Query {
    pub fn utxo_set_point_latest(&self) -> Result<UtxoSetPoint> {
        let pin = self.pin_safe_lengths()?;
        let end = self.utxo_set_end(&pin)?;
        if end == 0 {
            return Err(Error::NotFound("No published UTXO set".into()));
        }
        let last = Height::from(end - 1);
        self.utxo_set_point(last, last, false)
    }

    pub fn utxo_set_point_height(&self, height: Height) -> Result<UtxoSetPoint> {
        let pin = self.pin_safe_lengths()?;
        check_bound(&pin, height, pin.lengths().height)?;
        if usize::from(height) >= self.utxo_set_end(&pin)? {
            return Err(Error::NotFound(format!("Block {height} is not published")));
        }
        self.utxo_set_point(height, height, pin.lengths().is_settled(height))
    }

    pub fn utxo_set_point_date(&self, date: Date) -> Result<UtxoSetPoint> {
        let pin = self.pin_safe_lengths()?;
        let end = self.utxo_set_end(&pin)?;
        let day = Day1::try_from(date)?;
        let (heights, settled) = self.day_heights(day, end, &pin)?;
        self.utxo_set_point(
            Height::from(heights.start),
            Height::from(heights.end - 1),
            settled,
        )
    }

    /// The set after block `point.last`.
    pub fn utxo_set(&self, point: &UtxoSetPoint) -> Result<UtxoSet> {
        let pin = self.pin_safe_lengths()?;
        let view = self.plugins().utxo_set.view()?;
        let reader = view.reader(usize::from(pin.lengths().height))?;
        let end = point.end_within(reader.len())?;
        let state = reader.state_at(end).map_err(updating)?;
        if state.hash() != *point.hash {
            return Err(Error::StateUpdating);
        }
        let amounts = state.amounts();
        let total = state.total();
        Ok(UtxoSet {
            height: point.last,
            hash: point.hash,
            date: point.date,
            count: total.count,
            supply: btc(total.sats),
            origins: UtxoOrigins {
                count: amounts.iter().map(|amount| amount.count).collect(),
                supply: amounts.iter().map(|amount| btc(amount.sats)).collect(),
            },
        })
    }

    /// What blocks `point.first` through `point.last` changed, read from their journals alone.
    pub fn utxo_set_diff(&self, point: &UtxoSetPoint) -> Result<UtxoSetDiff> {
        let pin = self.pin_safe_lengths()?;
        let view = self.plugins().utxo_set.view()?;
        let reader = view.reader(usize::from(pin.lengths().height))?;
        let end = point.end_within(reader.len())?;
        let first = usize::from(point.first);
        let mut diffs = reader.diffs(first).map_err(updating)?;
        let mut created = UtxoChanges::default();
        let mut spent = BTreeMap::<u32, (u64, u64)>::new();
        for height in first..end {
            let diff = diffs.advance().map_err(updating)?.data()?;
            created.height.push(Height::from(height));
            created.count.push(diff.created.count);
            created.supply.push(btc(diff.created.sats));
            for (origin, amount) in diff.removed() {
                let (count, sats) = spent.entry(origin).or_default();
                *count += amount.count;
                *sats += amount.sats;
            }
        }
        if diffs.hash() != Some(*point.hash) {
            return Err(Error::StateUpdating);
        }
        let mut removed = UtxoChanges::default();
        for (origin, (count, sats)) in spent {
            removed.height.push(Height::from(origin));
            removed.count.push(count);
            removed.supply.push(btc(sats));
        }
        Ok(UtxoSetDiff {
            first: point.first,
            last: point.last,
            hash: point.hash,
            date: point.date,
            created,
            spent: removed,
        })
    }

    /// Blocks published both by the chain and by the UTXO set's history.
    fn utxo_set_end(&self, pin: &ReadPin) -> Result<usize> {
        let view = self.plugins().utxo_set.view()?;
        Ok(view.reader(usize::from(pin.lengths().height))?.len())
    }

    fn utxo_set_point(&self, first: Height, last: Height, settled: bool) -> Result<UtxoSetPoint> {
        let hash = self
            .indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(last)
            .data()?;
        let timestamp = self
            .plugins()
            .mappings
            .timestamp
            .monotonic
            .collect_one(last)
            .data()?;
        Ok(UtxoSetPoint {
            first,
            last,
            hash,
            date: Date::from(timestamp),
            settled,
        })
    }
}

/// A rewind racing a replay hands it another chain's rows, which the journals reject as invalid
/// data: the state is updating.
fn updating(error: std::io::Error) -> Error {
    if error.kind() == std::io::ErrorKind::InvalidData {
        Error::StateUpdating
    } else {
        error.into()
    }
}
