//! One model-based end-to-end test: both producers, the history and its readers, against a plain
//! in-memory chain replayed from genesis.
use crate::{Amount, BlockDiff, Creations, History, SNAPSHOT_INTERVAL, Spends, State, View};
use std::{
    collections::BTreeMap,
    fs,
    io::{Error, Result},
    path::{Path, PathBuf},
};

const STEPS: usize = 600;

type Hash = [u8; 32];
type Versions = (u64, u64);
/// A published snapshot's identity: length, hash of its last block, producer versions.
type Snap = (usize, Hash, Versions);

struct Rng(u64);
impl Default for Rng {
    fn default() -> Self {
        Self(0x9E37_79B9_7F4A_7C15)
    }
}
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

struct Block {
    hash: Hash,
    created: Amount,
    spent: Vec<(u32, Amount)>,
    correction: Option<(u32, Amount)>,
}

struct Stores {
    spends: Spends,
    creations: Creations,
    history: History,
}
impl Stores {
    fn open(root: &Path) -> Result<Self> {
        Ok(Self {
            spends: Spends::open(&root.join("inputs"))?,
            creations: Creations::open(&root.join("outputs"))?,
            history: History::open(&root.join("utxo_set"))?,
        })
    }
}

#[derive(Default)]
struct Model {
    rng: Rng,
    blocks: Vec<Block>,
    /// The indexer's chain (block ids) with the state and per-block totals it reaches.
    chain: Vec<usize>,
    amounts: Vec<Amount>,
    totals: Vec<Amount>,
    /// The blocks each producer committed: spends, creations.
    made: [Vec<usize>; 2],
    versions: Versions,
    /// The UTXO Set's persisted height.
    consumer: usize,
    /// What the history published, and the state its writer keeps warm.
    tip: Option<Snap>,
    prev: Option<Snap>,
    retained: BTreeMap<usize, (Hash, Versions)>,
    live: Option<Snap>,
    /// Analytical state resumed by cursor across updates, like the age and URPD consumers'.
    resident: Option<State>,
    /// Open read-only views and the ceiling each reads at.
    views: Vec<(View, usize)>,
}
impl Model {
    fn hash(&self, chain: &[usize], len: usize) -> Hash {
        if len == 0 {
            [0; 32]
        } else {
            self.blocks[chain[len - 1]].hash
        }
    }
    fn snap(&self, len: usize) -> Snap {
        (len, self.hash(&self.chain, len), self.versions)
    }
    /// The state after `chain`: its amounts and total.
    fn walk(&self, chain: &[usize]) -> (Vec<Amount>, Amount) {
        let mut amounts = Vec::with_capacity(chain.len());
        let mut total = Amount::default();
        for &id in chain {
            let block = &self.blocks[id];
            amounts.push(block.created);
            total = total + block.created;
            for &(origin, amount) in block.spent.iter().chain(&block.correction) {
                amounts[origin as usize] = amounts[origin as usize] - amount;
                total = total - amount;
            }
        }
        (amounts, total)
    }
    /// Removes part of what `origin` holds now: spending its last output takes all its sats.
    fn take(&mut self, origin: usize) -> Option<Amount> {
        let held = self.amounts[origin];
        if held.count == 0 {
            return None;
        }
        let count = 1 + self.rng.below(held.count.min(3) as usize) as u64;
        let sats = if count == held.count {
            held.sats
        } else {
            self.rng.next() % (held.sats + 1)
        };
        self.amounts[origin] = held - Amount { sats, count };
        Some(Amount { sats, count })
    }
    /// Mines a block: zero-sat outputs, same-block and old spends, and BIP30-like overwrites.
    fn grow(&mut self) {
        let height = self.chain.len();
        let count = self.rng.below(4) as u64;
        let sats = if count > 0 && self.rng.chance(90) {
            self.rng.next() >> 24
        } else {
            0
        };
        let created = Amount { sats, count };
        self.amounts.push(created);
        let mut spent: Vec<(u32, Amount)> = Vec::new();
        for _ in 0..self.rng.below(5) {
            let origin = if self.rng.chance(50) {
                height.saturating_sub(self.rng.below(30))
            } else {
                self.rng.below(height + 1)
            };
            if !spent.iter().any(|&(o, _)| o as usize == origin)
                && let Some(amount) = self.take(origin)
            {
                spent.push((origin as u32, amount));
            }
        }
        let correction = if height > 0 && self.rng.chance(5) {
            let origin = self.rng.below(height);
            self.take(origin).map(|amount| (origin as u32, amount))
        } else {
            None
        };
        let removed = spent
            .iter()
            .chain(&correction)
            .fold(Amount::default(), |sum, &(_, amount)| sum + amount);
        let total = self.totals.last().copied().unwrap_or_default();
        self.totals.push(total + created - removed);
        let mut hash = [0; 32];
        for chunk in hash.chunks_mut(8) {
            chunk.copy_from_slice(&self.rng.next().to_le_bytes());
        }
        self.chain.push(self.blocks.len());
        self.blocks.push(Block {
            hash,
            created,
            spent,
            correction,
        });
    }
    /// The indexer's next update: new blocks, a catch-up sync, a reorg, a deep rewind, or a
    /// producer version bump (the producers validate versions first, rebuilding from genesis).
    /// Returns whether the chain rewound.
    fn change_chain(&mut self, s: &mut Stores) -> Result<bool> {
        let len = self.chain.len();
        let roll = self.rng.below(100);
        let fork = match roll {
            0..=14 => len.saturating_sub(1),
            15..=24 => len.saturating_sub(2 + self.rng.below(5)),
            25..=27 => self.rng.below(len + 1),
            _ => len,
        };
        let grow = match roll {
            _ if len < SNAPSHOT_INTERVAL => 300 + self.rng.below(3_000),
            28..=32 if len < 11_000 => 300 + self.rng.below(3_000),
            _ => len - fork + self.rng.below(4),
        };
        if roll == 33 {
            let p = self.rng.below(2);
            if p == 0 {
                self.versions.0 += 1;
            } else {
                self.versions.1 += 1;
            }
            self.made[p].clear();
            s.spends.validate_version(self.versions.0)?;
            s.creations.validate_version(self.versions.1)?;
            self.consumer = 0;
            self.views.clear();
        }
        if fork < len {
            self.chain.truncate(fork);
            self.totals.truncate(fork);
            self.amounts = self.walk(&self.chain).0;
            self.consumer = self.consumer.min(fork);
            self.views.retain(|&(_, ceiling)| ceiling <= fork);
        }
        for _ in 0..grow {
            self.grow();
        }
        Ok(fork < len)
    }
    /// One producer's update: rewind to the indexer's chain, push and commit (sometimes midway).
    /// Returns whether it quit before its final commit.
    fn sync(&mut self, s: &mut Stores, p: usize) -> Result<bool> {
        let common = self.made[p]
            .iter()
            .zip(&self.chain)
            .take_while(|(a, b)| a == b)
            .count();
        if p == 0 {
            s.spends.truncate(common)?;
        } else {
            s.creations.truncate(common)?;
        }
        self.made[p].truncate(common);
        let quit = self.rng.chance(4);
        let end = if quit {
            common + self.rng.below(self.chain.len() - common + 1)
        } else {
            self.chain.len()
        };
        for height in common..end {
            let block = &self.blocks[self.chain[height]];
            if p == 0 {
                s.spends.push(block.hash, block.spent.iter().copied())?;
            } else {
                s.creations
                    .push(block.hash, block.created, block.correction)?;
            }
            if self.rng.chance(2) || height + 1 == end && !quit {
                if p == 0 {
                    s.spends.commit()?;
                } else {
                    s.creations.commit()?;
                }
                let n = self.made[p].len();
                self.made[p].extend_from_slice(&self.chain[n..=height]);
            }
        }
        Ok(quit)
    }
    /// The UTXO Set's update through both producers, sometimes rebuilding from genesis (its own
    /// version bump) and sometimes with a visit failing after a retained checkpoint was written.
    fn advance(&mut self, s: &mut Stores) -> Result<()> {
        let end = self.made[0].len().min(self.made[1].len());
        let start = if self.rng.chance(2) { 0 } else { self.consumer };
        self.views.retain(|&(_, ceiling)| ceiling <= start);
        let checkpoint = start.max(1).next_multiple_of(SNAPSHOT_INTERVAL);
        let fail = (checkpoint < end && self.rng.chance(50))
            .then(|| checkpoint + self.rng.below(end - checkpoint));
        let mut visits = Vec::new();
        let result = s
            .history
            .advance(start, end, &s.spends, &s.creations, |height, total| {
                if Some(height) == fail {
                    return Err(Error::other("visit failed"));
                }
                visits.push((height, total));
                Ok(())
            });
        assert_eq!(result.is_ok(), fail.is_none(), "{result:?}");
        let reached = fail.unwrap_or(end);
        assert!(
            visits
                .into_iter()
                .eq((start..reached).map(|h| (h, self.totals[h]))),
            "visits {start}..{reached}"
        );
        // What the history publishes: an unchanged state at its newest snapshot is left alone.
        let tip = [self.tip, self.prev]
            .into_iter()
            .flatten()
            .filter(|tip| tip.2 == self.versions)
            .max_by_key(|tip| tip.0);
        let retained = self
            .retained
            .last_key_value()
            .filter(|(_, (_, versions))| *versions == self.versions)
            .map(|(&len, &(hash, versions))| (len, hash, versions));
        let latest = match (tip, retained) {
            (Some(tip), Some(retained)) if retained.0 > tip.0 => Some(retained),
            (tip, retained) => tip.or(retained),
        };
        if start == end && (self.live == Some(self.snap(start)) || latest == Some(self.snap(end))) {
            self.live = Some(self.snap(end));
            self.consumer = end;
            return Ok(());
        }
        self.retained.retain(|&len, _| len <= start);
        for tip in [&mut self.tip, &mut self.prev] {
            if tip.is_some_and(|tip| tip.0 > start) {
                *tip = None;
            }
        }
        for len in (checkpoint..=reached).step_by(SNAPSHOT_INTERVAL) {
            let (_, hash, versions) = self.snap(len);
            self.retained.entry(len).or_insert((hash, versions));
        }
        self.live = None;
        if fail.is_none() {
            self.prev = self.tip.or(self.prev);
            self.tip = Some(self.snap(end));
            self.live = self.tip;
            self.consumer = end;
        }
        Ok(())
    }
    /// Whether both producers carry this snapshot.
    fn carried(&self, (len, hash, versions): Snap) -> bool {
        versions == self.versions
            && self
                .made
                .iter()
                .all(|made| len <= made.len() && hash == self.hash(made, len))
    }
    /// The end `History::reader` publishes: the newest carried tip, or a newer carried retained
    /// snapshot.
    fn reader_end(&self) -> usize {
        let limit = self.made[0].len().min(self.made[1].len());
        let tip = [self.tip, self.prev]
            .into_iter()
            .flatten()
            .filter(|&tip| self.carried(tip))
            .map(|tip| tip.0)
            .max()
            .unwrap_or(0);
        self.retained
            .range(..=limit)
            .rev()
            .take_while(|(len, _)| **len > tip)
            .find(|&(&len, &(hash, versions))| self.carried((len, hash, versions)))
            .map_or(tip, |(&len, _)| len)
    }
    fn assert_state(&self, state: &State, chain: &[usize]) {
        let walked;
        let (amounts, total) = if chain == self.chain {
            (
                &self.amounts,
                self.totals.last().copied().unwrap_or_default(),
            )
        } else {
            walked = self.walk(chain);
            (&walked.0, walked.1)
        };
        let differs = state
            .amounts()
            .iter()
            .zip(amounts)
            .position(|(a, b)| a != b);
        assert!(
            state.amounts() == amounts.as_slice(),
            "state of {} blocks: {} amounts, first difference at {differs:?}",
            chain.len(),
            state.len()
        );
        assert_eq!(state.total(), total);
        assert_eq!(state.hash(), self.hash(chain, chain.len()));
    }
    fn assert_diff(&self, diff: &BlockDiff, id: usize) {
        let block = &self.blocks[id];
        assert_eq!(
            (diff.hash, diff.created, diff.correction()),
            (block.hash, block.created, block.correction)
        );
        assert!(diff.spent().eq(block.spent.iter().copied()));
    }
    /// Every read path against the model: the history's reader (tips, `prev` during a reorg,
    /// retained snapshots, replays), a resident state's cursor, raw diffs, and the open views.
    fn check(&mut self, s: &Stores) -> Result<()> {
        assert_eq!(
            (s.spends.end(), s.creations.end()),
            (self.made[0].len(), self.made[1].len())
        );
        let reader = s.history.reader(&s.spends, &s.creations)?;
        let end = self.reader_end();
        assert_eq!((reader.len(), reader.versions()), (end, self.versions));
        // Both producers carry the same blocks through the reader's end.
        let chain = &self.made[0][..end];
        self.assert_state(&reader.state_at(end)?, chain);
        let h = self.rng.below(end + 1);
        self.assert_state(&reader.state_at(h)?, &chain[..h]);

        let mut state = match self.resident.take() {
            Some(state) => state,
            None => reader.state_at(self.rng.below(end + 1))?,
        };
        let carried = state.len() <= end && state.hash() == self.hash(chain, state.len());
        assert_eq!(reader.matches(&state)?, carried);
        if !carried {
            assert!(reader.cursor(&mut state).is_err());
            state = reader.state_at(self.rng.below(end + 1))?;
        }
        let mut height = state.len();
        let mut cursor = reader.cursor(&mut state)?;
        while let Some(diff) = cursor.advance()? {
            self.assert_diff(&diff, chain[height]);
            height += 1;
        }
        assert_eq!(height, end);
        self.assert_state(cursor.state(), chain);
        // A match cached for consumers resuming from one prefix never vouches for another.
        let forged = State::new(state.amounts().to_vec(), [!0; 32])?;
        assert!(reader.matches(&state)? && reader.matches(&forged)? == state.is_empty());
        if !self.rng.chance(10) {
            self.resident = Some(state);
        }

        let first = self.rng.below(end + 1);
        let last = end.min(first + 50);
        let mut diffs = reader.diffs(first)?;
        for &id in &chain[first..last] {
            self.assert_diff(&diffs.advance()?.unwrap(), id);
        }
        assert_eq!(diffs.hash(), (last > first).then(|| self.hash(chain, last)));

        for (view, ceiling) in &self.views {
            let reader = view.reader(*ceiling)?;
            assert_eq!(reader.len(), *ceiling);
            let h = self.rng.below(ceiling + 1);
            self.assert_state(&reader.state_at(h)?, &self.chain[..h]);
        }
        Ok(())
    }
}

/// Removes the test's directory, also when it fails.
struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn model() -> Result<()> {
    let dir = Dir(std::env::temp_dir().join(format!("statedb-model-{}", std::process::id())));
    let root = &dir.0;
    let _ = fs::remove_dir_all(root);
    let mut s = Stores::open(root)?;
    assert!(Spends::open(&root.join("inputs")).is_err(), "second writer");
    assert!(
        History::open(&root.join("utxo_set")).is_err(),
        "second writer"
    );
    let row = Amount { sats: 1, count: 1 };
    assert!(
        s.spends.push([0; 32], [(1, row)]).is_err(),
        "spend above its block"
    );
    assert!(
        s.creations.push([0; 32], row, Some((0, row))).is_err(),
        "overwrite of its own block"
    );
    let mut m = Model::default();
    m.check(&s)?;
    for _ in 0..STEPS {
        let reorg = m.change_chain(&mut s)?;
        for p in 0..2 {
            // A soft quit mid-update: the next writer discards the uncommitted tail.
            if m.sync(&mut s, p)? || m.rng.chance(5) {
                drop(s);
                s = Stores::open(root)?;
                m.live = None;
            }
            // Readers between the producers, and before the consumer of a reorg: they read `prev`.
            if m.rng.chance(15) || reorg && p == 1 {
                m.check(&s)?;
            }
        }
        m.advance(&mut s)?;
        m.check(&s)?;
        if m.views.len() < 3 && m.rng.chance(15) {
            let ceiling = if m.rng.chance(50) {
                m.consumer
            } else {
                m.rng.below(m.consumer + 1)
            };
            let view = View::open(
                &root.join("utxo_set"),
                &root.join("inputs"),
                &root.join("outputs"),
            )?;
            m.views.push((view, ceiling));
        }
        if m.rng.chance(3) {
            m.views.pop();
        }
    }
    drop(s);
    m.live = None;
    m.check(&Stores::open(root)?)?;
    Ok(())
}
