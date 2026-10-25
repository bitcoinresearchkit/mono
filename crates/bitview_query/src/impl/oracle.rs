use std::{collections::HashMap, sync::Arc};

use bitview_plugin_price::{block_payments, block_tally, feed_blocks_for_warmup, feed_blocks_with};
use bitview_primitives::Lengths;
use bitview_types::{OracleBlock, OracleBlockPayments, OraclePayments};
use brk_oracle::{
    Config, Oracle, ROUND_DOLLARS, START_HEIGHT_FAST, START_HEIGHT_SLOW, Search, Tally,
    bin_to_cents,
};
use brk_types::{BlockHash, Dollars, Height};
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, ReadPin, Result, check_bound};

/// Blocks in one chunk of payments: a day's. Chunks start at its multiples, so their requests are
/// few and fixed, and a settled one never changes.
pub const ORACLE_PAYMENTS_CHUNK: usize = 144;

/// A payments bucket's width, in oracle bins: a tenth, 0.115% of price.
pub const PAYMENT_STEP: f64 = 0.1;

impl Query {
    /// The oracle at confirmed `height`: every candidate bin it scored from the block before's
    /// answer, the bin it picked (the block's price), the smoothed histogram it scored and the
    /// block's outputs as its filter judged them. After [`START_HEIGHT_FAST`] (before, the slow
    /// regime's shape profile learns from each pick). Also whether `height` is settled.
    pub fn oracle_block(&self, height: usize) -> Result<(OracleBlock, bool)> {
        if height <= START_HEIGHT_FAST {
            return Err(Error::OutOfRange(
                format!("the oracle's blocks: after height {START_HEIGHT_FAST}").into(),
            ));
        }
        let pin = self.pin_safe_lengths()?;
        check_bound(&pin, height, pin.lengths().height.to_usize())?;
        let safe = self.check_histogram_height(height, pin.lengths())?;
        // (The window's histogram doesn't depend on the seed: the answers do, read as stored.)
        let prev = self.seed_bin_at(height - 1)?;
        let oracle = self.warm_oracle(prev, height + 1, &safe)?;
        let search = oracle
            .search(prev)
            .ok_or_else(|| Error::NotFound(format!("no oracle reading at height {height}")))?;
        let bin = self.seed_bin_at(height)?;
        let tally = block_tally(self.indexer(), height, Some(&safe))?;
        let hash = self.block_hash_at(height)?;
        Ok((
            reading(height, hash, search, bin, &oracle, tally, false),
            pin.lengths().is_settled(Height::from(height)),
        ))
    }

    /// The chunk's blocks' payments (from `start`, a multiple of [`ORACLE_PAYMENTS_CHUNK`] past
    /// [`START_HEIGHT_FAST`], else an invalid request), each read as each of the oracle's
    /// round-dollar amounts at its exact amount in sats, counted by the price each reading implies,
    /// in buckets of [`PAYMENT_STEP`] bins over the block's search window. The chunk at the tip
    /// holds the blocks so far. Also whether the chunk is complete and settled.
    pub fn oracle_payments_chunk(&self, start: usize) -> Result<(Arc<OraclePayments>, bool)> {
        if !start.is_multiple_of(ORACLE_PAYMENTS_CHUNK) {
            let first = START_HEIGHT_FAST / ORACLE_PAYMENTS_CHUNK * ORACLE_PAYMENTS_CHUNK
                + ORACLE_PAYMENTS_CHUNK;
            return Err(Error::InvalidParam(format!(
                "the oracle's payments: `start` is a multiple of {ORACLE_PAYMENTS_CHUNK}, as {}",
                (start / ORACLE_PAYMENTS_CHUNK * ORACLE_PAYMENTS_CHUNK).max(first)
            )));
        }
        if start <= START_HEIGHT_FAST {
            return Err(Error::OutOfRange(
                format!("the oracle's payments: after height {START_HEIGHT_FAST}, from 508,032")
                    .into(),
            ));
        }
        let pin = self.pin_safe_lengths()?;
        check_bound(&pin, start, pin.lengths().height.to_usize())?;
        let end = (start + ORACLE_PAYMENTS_CHUNK).min(self.histogram_bound(&pin.lengths()));
        if end <= start {
            return Err(Error::NotFound(format!(
                "no oracle payments from height {start} yet"
            )));
        }
        // (Replayed once: a settled chunk for good, the others once per publication; either only
        // while its last block is still on the chain, which a rollback past settlement replaces.)
        let last = self.block_hash_at(end - 1)?;
        let hit = self.0.payment_chunks.lock().get(start, pin.revision());
        if let Some(hit) = hit.filter(|(payments, _)| payments.hash == last) {
            return Ok(hit);
        }
        let range = start..end;
        let safe = self.check_histogram_height(end - 1, pin.lengths())?;
        // (Each block's search window, from the block before's answer as stored, as
        // `oracle_block` finds it.)
        let prevs = (start - 1..end - 1)
            .map(|height| self.seed_bin_at(height))
            .collect::<Result<Vec<_>>>()?;
        let mut oracle = self.warm_oracle(prevs[0], start, &safe)?;
        let mut windows = HashMap::with_capacity(range.len());
        feed_blocks_with(
            &mut oracle,
            self.indexer(),
            range.clone(),
            Some(&safe),
            |height, oracle, _| {
                if let Some(search) = oracle.search(prevs[height - start]) {
                    windows.insert(height, (search.first_bin, search.scores.len()));
                }
            },
        )?;
        let per_bin = (1.0 / PAYMENT_STEP).round() as i64;
        let amounts = ROUND_DOLLARS.map(f64::log10);
        let mut blocks = Vec::with_capacity(range.len());
        block_payments(
            self.indexer(),
            range.clone(),
            Some(&safe),
            |height, payments| {
                let Some(&(first_bin, bins)) = windows.get(&height) else {
                    return;
                };
                // (A bin spans half a bin either side of it: the window's first bucket starts there.)
                let first = first_bin as i64 * per_bin - per_bin / 2;
                let mut counts = vec![0u32; bins * per_bin as usize];
                for sats in payments {
                    let at = ((*sats as f64).log10() + 2.0) * 200.0 / PAYMENT_STEP;
                    for amount in amounts {
                        // (A reading on a bucket's edge, as $25 in 25,000 sats is $100,000 exactly, can land
                        // a hair under it in floating point: within a millionth of a bucket is on it.)
                        let at = at - amount * 200.0 / PAYMENT_STEP;
                        let edge = at.round();
                        let bucket = if (at - edge).abs() < 1e-6 {
                            edge
                        } else {
                            at.floor()
                        } as i64
                            - first;
                        if let Some(count) =
                            usize::try_from(bucket).ok().and_then(|b| counts.get_mut(b))
                        {
                            *count += 1;
                        }
                    }
                }
                blocks.push(OracleBlockPayments {
                    height: Height::from(height),
                    first_bucket: first as u32,
                    counts,
                });
            },
        )?;
        let settled =
            range.len() == ORACLE_PAYMENTS_CHUNK && pin.lengths().is_settled(Height::from(end - 1));
        let payments = Arc::new(OraclePayments {
            hash: last,
            step: PAYMENT_STEP,
            blocks,
        });
        self.0
            .payment_chunks
            .lock()
            .put(start, pin.revision(), settled, Arc::clone(&payments));
        Ok((payments, settled))
    }

    /// The oracle now, from the live oracle the server keeps warm (no replay): with the mempool's
    /// outputs blended in as the next block, its reading of it from the tip's answer; without a
    /// mempool publication at this tip, the tip block's own. One snapshot. After
    /// [`START_HEIGHT_FAST`], as [`Self::oracle_block`].
    pub fn live_block(&self) -> Result<OracleBlock> {
        let live = self.live_oracle()?;
        let (oracle, height, prev) = match live.blended_from {
            Some(prev) => (live.oracle, live.next, prev),
            None => (live.oracle, live.next - 1, live.tip_prev),
        };
        // (The mempool's outputs, from the same publication as its histogram; without it, the tip's.)
        let tally = match live.mempool {
            Some(tally) => tally,
            None => block_tally(self.indexer(), height, Some(&live.pin.lengths()))?,
        };
        if height <= START_HEIGHT_FAST {
            return Err(Error::OutOfRange(
                format!("the oracle's blocks: after height {START_HEIGHT_FAST}").into(),
            ));
        }
        let search = oracle
            .search(prev)
            .ok_or_else(|| Error::NotFound("no oracle reading at the tip".to_owned()))?;
        let hash = live.pin.tip();
        Ok(reading(
            height,
            hash,
            search,
            oracle.ref_bin(),
            &oracle,
            tally,
            live.mempool.is_some(),
        ))
    }

    pub fn live_price(&self) -> Result<Dollars> {
        Ok(self.live_oracle()?.oracle.price_dollars())
    }

    /// The live tip oracle: the committed base, with the published pool's mempool outputs
    /// blended in as a final slot when a mempool is configured.
    fn live_oracle(&self) -> Result<LiveOracle> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let tip = pin.tip();
        // Capture the completed mempool publication while the confirmed anchor
        // is held stable, before doing potentially expensive window work. A pool
        // not yet at this tip has nothing to blend.
        let live = match mempool.map(|mempool| mempool.live_payments(&tip)) {
            Some(Ok(payments)) => Some(payments),
            None | Some(Err(brk_error::Error::StateUpdating)) => None,
            Some(Err(error)) => return Err(error.into()),
        };
        let safe = pin.lengths();
        let height = safe.height.to_usize();
        let last = height
            .checked_sub(1)
            .ok_or_else(|| Error::NotFound("oracle prices not yet computed".to_owned()))?;
        let seed_bin = self.seed_bin_at(last)?;
        let mut oracle = self
            .0
            .live_oracle
            .get_or_try_init(pin.revision(), || self.warm_oracle(seed_bin, height, &safe))?;
        let (blended_from, mempool) = live
            .map(|(histogram, tally)| {
                let from = oracle.ref_bin();
                oracle.process_histogram(&histogram);
                (from, tally)
            })
            .unzip();
        // (Under the same pin as the oracle: a reorganization can't put another fork's answer here.)
        let tip_prev = self.seed_bin_at(last.saturating_sub(1))?;
        Ok(LiveOracle {
            oracle,
            blended_from,
            mempool,
            next: height,
            tip_prev,
            pin,
        })
    }

    /// An oracle seeded at `seed_bin` and warmed by replaying the `window_size`
    /// committed blocks ending just before `end`. Reads are capped at `safe` so
    /// concurrent indexer writes past the cap stay invisible.
    fn warm_oracle(&self, seed_bin: f64, end: usize, safe: &Lengths) -> Result<Oracle> {
        let config = Config::for_height(end.saturating_sub(1));
        // Like the price plugin: the oracle's window never reaches below its first block.
        let start = end
            .saturating_sub(config.window_size)
            .max(START_HEIGHT_SLOW.min(end));
        Ok(Oracle::from_checkpoint(seed_bin, config, |o| {
            feed_blocks_for_warmup(o, self.indexer(), start..end, Some(safe))
        })?)
    }

    /// Seed bin for an oracle warm-up: the oracle's exact reference bin at `height`; `NotFound`
    /// before the oracle reaches it.
    fn seed_bin_at(&self, height: usize) -> Result<f64> {
        self.plugins()
            .price
            .oracle_bin
            .collect_one_at(height)
            .map(f64::from)
            .ok_or_else(|| Error::NotFound("oracle prices not yet computed".to_string()))
    }

    /// The hash of confirmed block `height`.
    fn block_hash_at(&self, height: usize) -> Result<BlockHash> {
        self.indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(Height::from(height))
            .data()
    }

    fn histogram_bound(&self, safe: &Lengths) -> usize {
        self.plugins()
            .price
            .spot
            .cents
            .height
            .len()
            .min(safe.height.to_usize())
    }

    /// `height < min(spot price len, safe height)` or 404.
    /// Returns the safe lengths so callers cap reads at the same bound.
    fn check_histogram_height(&self, height: usize, safe: Lengths) -> Result<Lengths> {
        let bound = self.histogram_bound(&safe);
        if height >= bound {
            return Err(Error::NotFound(format!(
                "no oracle reading at height {height} yet"
            )));
        }
        Ok(safe)
    }
}

/// A block's reading, for the wire: its search, the histogram `oracle` scored and the block's
/// outputs as the filter judged them.
fn reading(
    height: usize,
    hash: BlockHash,
    search: Search,
    bin: f64,
    oracle: &Oracle,
    tally: Tally,
    mempool: bool,
) -> OracleBlock {
    OracleBlock {
        height: Height::from(height),
        hash,
        mempool,
        price: bin_to_cents(bin).into(),
        bin,
        first_bin: search.first_bin as u16,
        scores: search.scores,
        histogram: oracle.ema().clone(),
        outputs: tally.into(),
    }
}

/// The live oracle and what its searches start from.
struct LiveOracle {
    oracle: Oracle,
    /// With the mempool blended in, the reference its search started from: the tip's answer.
    blended_from: Option<f64>,
    /// With the mempool blended in, its outputs as the filter judged them.
    mempool: Option<Tally>,
    /// The next block's height.
    next: usize,
    /// The tip block's previous answer: where its own search started.
    tip_prev: f64,
    /// The pin it was read under, held while it's used: no reorganization between its parts.
    pin: ReadPin,
}

/// Payment chunks already replayed: a settled one kept for good (it never changes), the others for
/// the publication they were read at; past [`Self::MAX`], the least recently used go first.
pub struct OracleChunks<T> {
    /// Each chunk by its start: the revision it was read at (none once settled), when it was last
    /// used, and the chunk.
    chunks: HashMap<usize, (Option<u64>, u64, Arc<T>)>,
    uses: u64,
}

impl<T> Default for OracleChunks<T> {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            uses: 0,
        }
    }
}

impl<T> OracleChunks<T> {
    const MAX: usize = 64;

    fn get(&mut self, start: usize, revision: u64) -> Option<(Arc<T>, bool)> {
        self.uses += 1;
        let (read_at, used, chunk) = self.chunks.get_mut(&start)?;
        let settled = match read_at {
            None => true,
            Some(read_at) if *read_at == revision => false,
            Some(_) => return None,
        };
        *used = self.uses;
        Some((Arc::clone(chunk), settled))
    }

    fn put(&mut self, start: usize, revision: u64, settled: bool, chunk: Arc<T>) {
        self.uses += 1;
        if self.chunks.len() >= Self::MAX
            && !self.chunks.contains_key(&start)
            && let Some(&unused) = self
                .chunks
                .iter()
                .min_by_key(|(_, (_, used, _))| *used)
                .map(|(start, _)| start)
        {
            self.chunks.remove(&unused);
        }
        self.chunks
            .insert(start, ((!settled).then_some(revision), self.uses, chunk));
    }
}
