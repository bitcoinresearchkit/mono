use crate::{columns, journal::Journal, journal_reader::JournalReader, util::invalid};
use std::{io::Result, ops::Range, path::Path};

/// Blocks between checkpoints: a restore reads the nearer one and at most half this many blocks'
/// changes.
const CHECKPOINT: usize = 250;

/// Sparse u64 cells keyed by u32, after every block: each block's signed changes, and every
/// nonzero cell each `CHECKPOINT` blocks. A record leads with its block's hash; checkpoint `i`
/// holds the cells after `i * CHECKPOINT` blocks, the first the empty set before genesis.
pub struct CellHistory {
    changes: Journal,
    checkpoints: Journal,
    buffer: Vec<u8>,
    keys: Vec<u32>,
    values: Vec<u64>,
}

impl CellHistory {
    pub fn open(path: &Path) -> Result<Self> {
        let mut this = Self::new(
            Journal::open(&path.join("changes"))?,
            Journal::open(&path.join("checkpoints"))?,
        );
        // Checkpoints are written and published first: a soft quit can leave some past the
        // changes, never fewer than they need.
        this.truncate(this.len())?;
        this.genesis()?;
        Ok(this)
    }
    pub fn open_reader(path: &Path) -> Result<Self> {
        Ok(Self::new(
            Journal::open_reader(&path.join("changes"))?,
            Journal::open_reader(&path.join("checkpoints"))?,
        ))
    }
    fn new(changes: Journal, checkpoints: Journal) -> Self {
        Self {
            changes,
            checkpoints,
            buffer: Vec::new(),
            keys: Vec::new(),
            values: Vec::new(),
        }
    }
    /// The blocks recorded.
    pub fn len(&self) -> usize {
        self.changes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn version(&self) -> u64 {
        self.changes.version
    }
    /// Starts over from genesis when the cells' sources changed.
    pub fn validate_version(&mut self, version: u64) -> Result<()> {
        self.changes.validate_version(version)?;
        self.checkpoints.validate_version(version)?;
        self.genesis()
    }
    fn genesis(&mut self) -> Result<()> {
        if self.checkpoints.len() == 0 {
            self.record([0; 32], [])?;
            self.checkpoints.push(&self.buffer)?;
            self.checkpoints.commit()?;
        }
        Ok(())
    }
    /// Keeps the first `len` blocks, all published.
    pub fn truncate(&mut self, len: usize) -> Result<()> {
        if len > self.len() {
            return Err(invalid("cell truncate past the end"));
        }
        // A journal truncate publishes what it keeps: the checkpoints go first, as in a commit.
        self.commit()?;
        self.changes.truncate(len)?;
        self.checkpoints
            .truncate((len / CHECKPOINT + 1).min(self.checkpoints.len()))
    }
    /// Appends the next block: its hash and its cell changes (ascending keys, none zero). When
    /// the block completes a checkpoint, `cells` lists every nonzero cell after it, by key.
    pub fn push<I: IntoIterator<Item = (u32, u64)>>(
        &mut self,
        hash: [u8; 32],
        changes: &[(u32, i64)],
        cells: impl FnOnce() -> I,
    ) -> Result<()> {
        debug_assert_eq!(self.checkpoints.len(), self.len() / CHECKPOINT + 1);
        if (self.len() + 1).is_multiple_of(CHECKPOINT) {
            self.record(hash, cells())?;
            self.checkpoints.push(&self.buffer)?;
        }
        self.record(hash, changes.iter().map(|&(key, change)| (key, zigzag(change))))?;
        self.changes.push(&self.buffer)
    }
    pub fn commit(&mut self) -> Result<()> {
        self.checkpoints.commit()?;
        self.changes.commit()
    }
    /// Packs one record into the buffer: the hash, the cell count, then keys and values as PCO.
    fn record(&mut self, hash: [u8; 32], cells: impl IntoIterator<Item = (u32, u64)>) -> Result<()> {
        self.keys.clear();
        self.values.clear();
        for (key, value) in cells {
            self.keys.push(key);
            self.values.push(value);
        }
        self.buffer.clear();
        self.buffer.extend_from_slice(&hash);
        self.buffer
            .extend_from_slice(&(self.keys.len() as u32).to_le_bytes());
        if !self.keys.is_empty() {
            columns::put_keys(&mut self.buffer, &mut self.keys)?;
            columns::put(&mut self.buffer, &self.values)?;
        }
        Ok(())
    }
    /// Hands `add` the cells after `end` blocks as (key, amount) additions to zeroed cells, from
    /// the nearer checkpoint within `ceiling` blocks: the one at or below `end`, then each later
    /// block's changes, or the next one, then the changes of the blocks before it taken back,
    /// newest first so that no cell passes below zero. Reads nothing past `ceiling`. Returns the
    /// hash of block `end - 1` (zeros before genesis).
    pub fn restore(
        &self,
        end: usize,
        ceiling: usize,
        mut add: impl FnMut(u32, i64) -> Result<()>,
    ) -> Result<[u8; 32]> {
        if end > ceiling || ceiling > self.len() {
            return Err(invalid("cells past the recorded blocks"));
        }
        let below = end / CHECKPOINT;
        let next = (below + 1) * CHECKPOINT;
        let above = next <= ceiling && next - end < end - below * CHECKPOINT;
        let at = below + usize::from(above);
        let (mut keys, mut values) = (Vec::new(), Vec::new());
        let mut checkpoints = JournalReader::new(&self.checkpoints, at + 1);
        let mut hash = decode(checkpoints.read(at)?.1, &mut keys, &mut values)?;
        for (&key, &value) in keys.iter().zip(&values) {
            add(key, i64::try_from(value).map_err(|_| invalid("cell exceeds i64"))?)?;
        }
        if above {
            let mut later = Vec::new();
            self.blocks(end..next, |_, changes| -> Result<()> {
                later.extend_from_slice(changes);
                Ok(())
            })?;
            later.iter().rev().try_for_each(|&(key, change)| add(key, -change))?;
            return self.hash(end);
        }
        self.blocks(below * CHECKPOINT..end, |block, changes| {
            hash = block;
            changes.iter().try_for_each(|&(key, change)| add(key, change))
        })?;
        Ok(hash)
    }
    /// Hands `block` each block of `range` in order: its hash and its changes by key.
    pub fn blocks<E: From<std::io::Error>>(
        &self,
        range: Range<usize>,
        mut block: impl FnMut([u8; 32], &[(u32, i64)]) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), E> {
        if range.end > self.len() {
            return Err(invalid("cells past the recorded blocks").into());
        }
        let (mut keys, mut values, mut changes) = (Vec::new(), Vec::new(), Vec::new());
        let mut reader = JournalReader::new(&self.changes, range.end);
        for height in range {
            let hash = decode(reader.read(height)?.1, &mut keys, &mut values)?;
            changes.clear();
            changes.extend(keys.iter().zip(&values).map(|(&key, &change)| (key, unzigzag(change))));
            block(hash, &changes)?;
        }
        Ok(())
    }
    /// The hash of block `end - 1` (zeros before genesis): which chain the cells after `end`
    /// blocks follow.
    pub fn hash(&self, end: usize) -> Result<[u8; 32]> {
        match end.checked_sub(1) {
            None => Ok([0; 32]),
            Some(last) if last < self.len() => {
                let mut reader = JournalReader::new(&self.changes, end);
                let record = reader.read(last)?.1;
                Ok(record.get(..32).ok_or_else(|| invalid("truncated cell record"))?.try_into().unwrap())
            }
            Some(_) => Err(invalid("cells past the recorded blocks")),
        }
    }
}

/// A record's hash, its keys and values decoded into `keys` and `values`.
fn decode(record: &[u8], keys: &mut Vec<u32>, values: &mut Vec<u64>) -> Result<[u8; 32]> {
    let n = columns::word(record, 32)? as usize;
    keys.resize(n, 0);
    values.resize(n, 0);
    let mut at = 36;
    if n > 0 {
        columns::take_keys(record, &mut at, keys)?;
        columns::take(record, &mut at, values)?;
    }
    if at != record.len() {
        return Err(invalid("trailing cell record bytes"));
    }
    Ok(record[..32].try_into().unwrap())
}

fn zigzag(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}
fn unzigzag(n: u64) -> i64 {
    ((n >> 1) as i64) ^ -((n & 1) as i64)
}
