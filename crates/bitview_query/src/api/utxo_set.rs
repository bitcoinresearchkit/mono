use std::collections::BTreeMap;

use bitview_plugin_indexer::SafeLengths;
use bitview_primitives::{Date, Day1};
use bitview_types::{UtxoChanges, UtxoOrigins, UtxoSet, UtxoSetDiff};
use brk_types::{Bitcoin, BlockHash, Height, Sats};
use statedb::{Reader, View};
use vecdb::ReadableVec;

use super::json;
use crate::{Error, OptionData, Query, ReadPin, Result, check_bound};

fn btc(sats: u64) -> Bitcoin {
    Bitcoin::from(Sats::new(sats))
}

/// The published blocks a point covers: one block, or a UTC day's blocks. It keeps the pin it was
/// resolved under and the history opened under it, which its bodies read: the tag and the body
/// come from one publication, and no file is opened twice.
pub struct UtxoSetPoint {
    pin: SafeLengths,
    view: View,
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

    /// A reader of the point's history through the blocks its pin publishes: the point lies
    /// within them.
    fn reader(&self) -> Result<Reader<'_>> {
        Ok(self.view.reader(usize::from(self.pin.lengths().height))?)
    }

    /// The set after block `last`, as the JSON of `UtxoSet`, written straight from its amounts.
    pub fn set_json(&self) -> Result<Vec<u8>> {
        // The reader goes before the body is written: it may share the updater's live state,
        // which the next block then would have to copy.
        let state = self.reader()?.state_at(usize::from(self.last) + 1)?;
        if state.hash() != *self.hash {
            return Err(Error::updating());
        }
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

    /// What blocks `first` through `last` changed, read from their journals alone, as the JSON
    /// of `UtxoSetDiff`.
    pub fn diff_json(&self) -> Result<Vec<u8>> {
        let reader = self.reader()?;
        let first = usize::from(self.first);
        let mut diffs = reader.diffs(first)?;
        let mut created = UtxoChanges::default();
        let mut spent = BTreeMap::<u32, (u64, u64)>::new();
        for height in first..=usize::from(self.last) {
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
        if diffs.hash() != Some(*self.hash) {
            return Err(Error::updating());
        }
        let mut removed = UtxoChanges::default();
        for (origin, (count, sats)) in spent {
            removed.height.push(Height::from(origin));
            removed.count.push(count);
            removed.supply.push(btc(sats));
        }
        Ok(serde_json::to_vec(&UtxoSetDiff {
            first: self.first,
            last: self.last,
            hash: self.hash,
            date: self.date,
            created,
            spent: removed,
        })?)
    }
}

impl Query {
    pub fn utxo_set_point_latest(&self) -> Result<UtxoSetPoint> {
        let pin = self.pin_safe_lengths()?;
        let (view, end) = self.utxo_set_view(&pin)?;
        if end == 0 {
            return Err(Error::NotFound("No published UTXO set".into()));
        }
        let last = Height::from(end - 1);
        self.utxo_set_point(pin, view, last, last, false)
    }

    pub fn utxo_set_point_height(&self, height: Height) -> Result<UtxoSetPoint> {
        let pin = self.pin_safe_lengths()?;
        check_bound(&pin, height, pin.lengths().height)?;
        let (view, end) = self.utxo_set_view(&pin)?;
        if usize::from(height) >= end {
            return Err(Error::NotFound(format!("Block {height} is not published")));
        }
        let settled = pin.lengths().is_settled(height);
        self.utxo_set_point(pin, view, height, height, settled)
    }

    pub fn utxo_set_point_date(&self, date: Date) -> Result<UtxoSetPoint> {
        let pin = self.pin_safe_lengths()?;
        let (view, end) = self.utxo_set_view(&pin)?;
        let day = Day1::try_from(date)?;
        let (heights, settled) = self.day_heights(day, end, &pin)?;
        self.utxo_set_point(
            pin,
            view,
            Height::from(heights.start),
            Height::from(heights.end - 1),
            settled,
        )
    }

    /// The UTXO set's history, opened once for a request, and the blocks both it and the chain
    /// publish under `pin`.
    fn utxo_set_view(&self, pin: &ReadPin) -> Result<(View, usize)> {
        let view = self.plugins().utxo_set.view()?;
        let end = view.len_within(usize::from(pin.lengths().height));
        Ok((view, end))
    }

    fn utxo_set_point(
        &self,
        pin: ReadPin,
        view: View,
        first: Height,
        last: Height,
        settled: bool,
    ) -> Result<UtxoSetPoint> {
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
            pin: pin.into_inner(),
            view,
            first,
            last,
            hash,
            date: Date::from(timestamp),
            settled,
        })
    }
}
