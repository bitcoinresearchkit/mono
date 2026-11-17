use std::collections::BTreeMap;

use bitview_primitives::{Date, Day1};
use bitview_types::{UtxoChanges, UtxoOrigins, UtxoSet, UtxoSetDiff};
use brk_types::{Bitcoin, BlockHash, Height, Sats};
use statedb::State;
use vecdb::ReadableVec;

use super::json;
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

    /// `state`, the set after the point, as the JSON of `UtxoSet`, written straight from its
    /// amounts.
    fn set_json(&self, state: &State) -> Result<Vec<u8>> {
        let (amounts, total) = (state.amounts(), state.total());
        let mut body = Vec::with_capacity(256 + 16 * amounts.len());
        json::open(&mut body, self.last, &self.hash, &self.date)?;
        json::key(&mut body, "count");
        json::append(&mut body, total.count, json::int);
        json::key(&mut body, "supply");
        json::append(&mut body, total.sats, json::decimal::<8>);
        body.extend_from_slice(b",\"origins\":{\"count\":");
        json::append_array(&mut body, amounts, |out, amount| json::int(out, 0, amount.count));
        json::key(&mut body, "supply");
        json::append_array(&mut body, amounts, |out, amount| json::decimal::<8>(out, 0, amount.sats));
        body.extend_from_slice(b"}}");
        debug_assert!(
            body == serde_json::to_vec(&UtxoSet {
                height: self.last,
                hash: self.hash,
                date: self.date,
                count: total.count,
                supply: btc(total.sats),
                origins: UtxoOrigins {
                    count: amounts.iter().map(|amount| amount.count).collect(),
                    supply: amounts.iter().map(|amount| btc(amount.sats)).collect(),
                },
            })?,
            "the UTXO set body is its schema's JSON"
        );
        Ok(body)
    }

    /// The point's exclusive end, once a reader of `history` blocks covers it.
    fn end_within(&self, history: usize) -> Result<usize> {
        let end = usize::from(self.last) + 1;
        if end > history {
            return Err(Error::updating());
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

    /// The set after block `point.last`, as the JSON of `UtxoSet`.
    pub fn utxo_set(&self, point: &UtxoSetPoint) -> Result<Vec<u8>> {
        let pin = self.pin_safe_lengths()?;
        let view = self.plugins().utxo_set.view()?;
        // The reader goes before the body is written: it may share the updater's live state,
        // which the next block then would have to copy.
        let state = {
            let reader = view.reader(usize::from(pin.lengths().height))?;
            reader.state_at(point.end_within(reader.len())?)?
        };
        if state.hash() != *point.hash {
            return Err(Error::updating());
        }
        point.set_json(&state)
    }

    /// What blocks `point.first` through `point.last` changed, read from their journals alone, as
    /// the JSON of `UtxoSetDiff`.
    pub fn utxo_set_diff(&self, point: &UtxoSetPoint) -> Result<Vec<u8>> {
        let pin = self.pin_safe_lengths()?;
        let view = self.plugins().utxo_set.view()?;
        let reader = view.reader(usize::from(pin.lengths().height))?;
        let end = point.end_within(reader.len())?;
        let first = usize::from(point.first);
        let mut diffs = reader.diffs(first)?;
        let mut created = UtxoChanges::default();
        let mut spent = BTreeMap::<u32, (u64, u64)>::new();
        for height in first..end {
            let diff = diffs.advance()?.data()?;
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
            return Err(Error::updating());
        }
        let mut removed = UtxoChanges::default();
        for (origin, (count, sats)) in spent {
            removed.height.push(Height::from(origin));
            removed.count.push(count);
            removed.supply.push(btc(sats));
        }
        Ok(serde_json::to_vec(&UtxoSetDiff {
            first: point.first,
            last: point.last,
            hash: point.hash,
            date: point.date,
            created,
            spent: removed,
        })?)
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
