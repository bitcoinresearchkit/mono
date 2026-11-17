use crate::{Amount, columns, journal::Journal, journal_reader::JournalReader, util::invalid};
use std::{io::Result, path::Path};

/// A spend record's fixed part: block hash, total spent, row count.
const HEADER: usize = 52;
/// The record format's revision, stored above the producer's 32-bit version: a new one rebuilds
/// the journal but leaves its version, and so what was derived from it, alone (0: fixed 16-byte
/// rows, 1: PCO columns).
const FORMAT: u64 = 1 << 32;

/// Decoded spend rows: origins, counts, sats.
pub(crate) type Rows = (Vec<u32>, Vec<u64>, Vec<u64>);

/// Per-block spent amounts grouped by the height of the outputs they spend.
pub struct Spends {
    journal: Journal,
    buffer: Vec<u8>,
    rows: Vec<(u32, Amount)>,
    keys: Vec<u32>,
    values: Vec<u64>,
}
impl Spends {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            journal: Journal::open(&path.join("spends"))?,
            buffer: Vec::new(),
            rows: Vec::new(),
            keys: Vec::new(),
            values: Vec::new(),
        })
    }
    pub(crate) fn open_reader(path: &Path) -> Result<Self> {
        Ok(Self {
            journal: Journal::open_reader(&path.join("spends"))?,
            buffer: Vec::new(),
            rows: Vec::new(),
            keys: Vec::new(),
            values: Vec::new(),
        })
    }
    pub fn end(&self) -> usize {
        self.journal.len()
    }
    /// The producer's version, without the format's.
    pub fn version(&self) -> u64 {
        self.journal.version & u64::from(u32::MAX)
    }
    /// Starts over from genesis unless the journal holds `version` (32 bits) in this format.
    pub fn validate_version(&mut self, version: u64) -> Result<()> {
        if version >> 32 != 0 {
            return Err(invalid("spend journal version exceeds 32 bits"));
        }
        self.journal.validate_version(FORMAT | version)
    }
    pub fn truncate(&mut self, height: usize) -> Result<()> {
        self.journal.truncate(height)
    }
    pub fn commit(&mut self) -> Result<()> {
        self.journal.commit()
    }
    /// Journals a block's spent rows, sorted by origin, as three PCO columns: origin gaps, counts
    /// and sats. The order is the format's: readers see the rows by origin, whatever order they
    /// came in.
    pub fn push(
        &mut self,
        hash: [u8; 32],
        rows: impl IntoIterator<Item = (u32, Amount)>,
    ) -> Result<()> {
        let mut total = Amount::default();
        self.rows.clear();
        for (origin, amount) in rows {
            if origin as usize > self.end() || amount.count == 0 {
                return Err(invalid("invalid spent origin"));
            }
            total = total.checked_add(amount)?;
            self.rows.push((origin, amount));
        }
        self.rows.sort_unstable_by_key(|&(origin, ..)| origin);
        self.buffer.clear();
        self.buffer.extend_from_slice(&hash);
        self.buffer.extend_from_slice(&total.encode());
        self.buffer.extend_from_slice(&(self.rows.len() as u32).to_le_bytes());
        if !self.rows.is_empty() {
            let rows = &self.rows;
            self.keys.clear();
            self.keys.extend(rows.iter().map(|&(origin, _)| origin));
            columns::put_keys(&mut self.buffer, &mut self.keys)?;
            let fields: [fn(&Amount) -> u64; 2] = [|a| a.count, |a| a.sats];
            for field in fields {
                self.values.clear();
                self.values.extend(rows.iter().map(|(_, amount)| field(amount)));
                columns::put(&mut self.buffer, &self.values)?;
            }
        }
        self.journal.push(&self.buffer)
    }
    pub(crate) fn cursor(&self, end: usize) -> JournalReader<'_> {
        JournalReader::new(&self.journal, end)
    }
    /// A record's block hash and total, its rows decoded into `rows` (origins, counts, sats);
    /// returns the row count.
    pub(crate) fn decode(record: &[u8], rows: &mut Rows) -> Result<([u8; 32], Amount, usize)> {
        if record.len() < HEADER {
            return Err(invalid("truncated spend record"));
        }
        let n = columns::word(record, 48)? as usize;
        let mut at = HEADER;
        if n > 0 {
            let (origins, counts, sats) = rows;
            if origins.len() < n {
                origins.resize(n, 0);
                counts.resize(n, 0);
                sats.resize(n, 0);
            }
            columns::take_keys(record, &mut at, &mut origins[..n])?;
            columns::take(record, &mut at, &mut counts[..n])?;
            columns::take(record, &mut at, &mut sats[..n])?;
        }
        if at != record.len() {
            return Err(invalid("trailing spend record bytes"));
        }
        Ok((
            record[..32].try_into().unwrap(),
            Amount::decode(&record[32..48]),
            n,
        ))
    }
    pub fn hash(&self, height: usize) -> Result<[u8; 32]> {
        let mut hash = [0; 32];
        self.journal.read_prefix(height, &mut hash)?;
        Ok(hash)
    }
}
