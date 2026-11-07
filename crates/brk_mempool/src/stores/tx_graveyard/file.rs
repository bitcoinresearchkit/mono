//! The graveyard on disk: every tombstone (its body with resolved prevouts, fee, first sighting,
//! chunk rate, removal and age, in microseconds: burials keep their order against the moment it
//! was last told) and the tip whose blocks it was told, with that moment, so the RBF trees, removal
//! reasons and first sightings outlive a restart; on a clean shutdown, the pool's records with it,
//! so a restart publishes after one cycle (what changed meanwhile) instead of reloading the pool
//! from Core. One lz4-compressed file, written whole to a temporary one, flushed, and renamed over
//! it: a crash mid-save leaves the last complete one (the periodic saves' carry no pool). What a
//! body determines (txid, wtxid, size, weight, sigops, policy vsize, its own RBF signaling) is
//! derived from it on reading, never taken from the file.

use std::{
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bitcoin::{
    Transaction as BitcoinTransaction,
    consensus::{deserialize, serialize},
    hashes::Hash,
};
use brk_error::{Error, Result};
use brk_types::{
    BlockHash, FeeRate, Sats, SigOps, Timestamp, Transaction, TxIn, TxOut, TxStatus, Txid, VSize,
    Vout, Weight,
};
use rapidhash::RapidHashSet;
use tracing::{info, warn};

use super::{TxGraveyard, TxTombstone};
use crate::{
    TxRemoval,
    state::TxEntry,
    stores::{ReadOnlyTxStore, tx_store::TxRecord},
};

/// Bumped when the layout changes: a file of another version is set aside, not read.
const MAGIC: &[u8; 8] = b"BVGRAVE4";
/// The most a file may hold, and say it decompresses to (its bodies are capped at 256 MiB): a
/// corrupt one allocates nothing past it.
const MAX_FILE_BYTES: usize = 1 << 30;

/// Where the graveyard is kept, and what the last save wrote.
pub struct GraveyardFile {
    path: PathBuf,
    saved: Mutex<Saved>,
}

#[derive(Default)]
struct Saved {
    /// When the graveyard last written was taken: an older one without the pool is never
    /// written over it.
    taken: Option<Instant>,
    revision: Option<u64>,
    /// Whether the last save carried the pool: the shutdown's, the last one.
    pool: bool,
}

impl GraveyardFile {
    pub fn new(path: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            path,
            saved: Mutex::default(),
        })
    }

    /// The graveyard last saved, its tombstones' ages carried over the downtime, and the pool a
    /// clean shutdown saved with it; empty ones without a file, or with one that doesn't read
    /// (set aside as `.unread`).
    pub fn load(&self) -> (TxGraveyard, Vec<TxRecord>) {
        let started = Instant::now();
        match self
            .read()
            .and_then(|bytes| bytes.map(|bytes| decode(&bytes)).transpose())
        {
            Ok(Some((graveyard, pool))) => {
                info!(
                    "Loaded mempool graveyard ({} tombstones) and pool ({} transactions) in {:?}",
                    graveyard.tombstones.len(),
                    pool.len(),
                    started.elapsed()
                );
                (graveyard, pool)
            }
            Ok(None) => Default::default(),
            Err(error) => {
                warn!(%error, path = %self.path.display(), "mempool graveyard: set aside");
                let _ = fs::rename(&self.path, self.path.with_extension("unread"));
                Default::default()
            }
        }
    }

    /// The file's bytes, none without one; one past [`MAX_FILE_BYTES`] isn't read.
    fn read(&self) -> Result<Option<Vec<u8>>> {
        match fs::metadata(&self.path) {
            Ok(metadata) if metadata.len() > MAX_FILE_BYTES as u64 => {
                Err(Error::Parse(format!("{} bytes", metadata.len())))
            }
            Ok(_) => Ok(Some(fs::read(&self.path)?)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Writes `graveyard`, taken at `taken`, whole, flushed, then renamed over the last save.
    /// `pending` gives the pool's record of a pending replacer, and its chunk rate: one is saved
    /// as a tombstone taken now (its tree's head), so a restart finds its tree whether it's still
    /// pending (it comes back, exhumed), confirmed meanwhile (the blocks since tell it mined) or
    /// gone. `pool`: the same publication's records, on a clean shutdown: that save always
    /// writes, and none without the pool writes after it. Any other writes nothing when a
    /// graveyard taken as late is already written.
    pub fn save<'a>(
        &self,
        graveyard: &TxGraveyard,
        pending: impl Fn(&Txid) -> Option<(&'a TxRecord, FeeRate)>,
        pool: Option<&ReadOnlyTxStore>,
        taken: Instant,
    ) -> Result<()> {
        let mut saved = self.saved.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if pool.is_none() && (saved.pool || saved.taken.is_some_and(|last| last >= taken)) {
            return Ok(());
        }
        let bytes = encode(graveyard, pending, pool);
        let dir = self.path.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(dir)?;
        let temporary = self.path.with_extension("saving");
        let mut file = File::create(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, &self.path)?;
        File::open(dir)?.sync_all()?;
        *saved = Saved {
            taken: Some(taken),
            revision: Some(graveyard.revision()),
            pool: pool.is_some(),
        };
        Ok(())
    }

    /// The revision the last save wrote: a failed one leaves it, so the next try writes again.
    pub fn saved_revision(&self) -> Option<u64> {
        self.saved
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .revision
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

fn encode<'a>(
    graveyard: &TxGraveyard,
    pending: impl Fn(&Txid) -> Option<(&'a TxRecord, FeeRate)>,
    pool: Option<&ReadOnlyTxStore>,
) -> Vec<u8> {
    let now = Instant::now();
    // Oldest first: loading buries them back in order. The pending heads of replacement trees
    // last, taken now (vanished, as far as a restart knows).
    let mut tombstones: Vec<TxTombstone> =
        graveyard.tombstones.iter().map(|(_, tomb)| tomb.clone()).collect();
    tombstones.sort_unstable_by_key(|tomb| tomb.removed_at);
    let mut heads: Vec<Txid> = tombstones
        .iter()
        .filter_map(|tomb| tomb.replaced_by().copied())
        .filter(|by| pending(by).is_some())
        .collect();
    heads.sort_unstable_by_key(|txid| bitcoin::Txid::from(*txid));
    heads.dedup();
    tombstones.extend(heads.iter().filter_map(|txid| {
        pending(txid).map(|(record, chunk_rate)| TxTombstone {
            record: record.clone(),
            chunk_rate,
            removal: TxRemoval::Vanished,
            removed_at: now,
        })
    }));
    let mut out = Writer::default();
    out.bytes(MAGIC);
    out.u64(unix_millis());
    match graveyard.told() {
        Some(tip) => {
            out.u8(1);
            out.bytes(&bitcoin::BlockHash::from(tip).to_byte_array());
            // Its age: what was buried before, the blocks since didn't displace.
            let at = graveyard.told_at().unwrap_or(now);
            out.u64(now.saturating_duration_since(at).as_micros() as u64);
        }
        None => out.u8(0),
    }
    out.u64(tombstones.len() as u64);
    for tomb in &tombstones {
        let age = now.saturating_duration_since(tomb.removed_at);
        out.u64(age.as_micros() as u64);
        match tomb.removal {
            TxRemoval::Mined => out.u8(0),
            TxRemoval::Vanished => out.u8(1),
            TxRemoval::Replaced { by } => {
                out.u8(2);
                out.bytes(&bitcoin::Txid::from(by).to_byte_array());
            }
        }
        out.u64(tomb.chunk_rate.milli());
        write_record(&mut out, &tomb.record);
    }
    match pool {
        Some(pool) => {
            out.u8(1);
            out.u64(pool.len() as u64);
            for (_, record) in pool.records() {
                write_record(&mut out, record);
            }
        }
        None => out.u8(0),
    }
    lz4_flex::compress_prepend_size(&out.0)
}

/// A record: fee, Core's policy vsize (a body with prevouts missing can't tell its sigops),
/// first sighting, signaling (what's inherited from an ancestor isn't in the body), the body,
/// each input's prevout when resolved.
fn write_record(out: &mut Writer, record: &TxRecord) {
    let entry = &record.entry;
    out.u64(u64::from(entry.fee));
    out.u64(u64::from(entry.vsize));
    out.u64(u64::from(entry.first_seen));
    out.u8(u8::from(entry.rbf));
    let tx = &record.tx;
    out.blob(&serialize(&BitcoinTransaction::from(tx.as_ref())));
    for txin in &tx.input {
        match &txin.prevout {
            Some(prevout) => {
                out.u8(1);
                out.u64(u64::from(prevout.value));
                out.blob(prevout.script_pubkey.as_bytes());
            }
            None => out.u8(0),
        }
    }
}

/// [`write_record`]'s, what the body determines derived from it.
fn read_record(input: &mut Reader) -> Result<TxRecord> {
    let fee = Sats::from(input.u64()?);
    if fee > Sats::MAX_MONEY {
        return Err(Error::Parse(format!("fee {fee}")));
    }
    let vsize = input.u64()?;
    let first_seen = Timestamp::from(input.u64()? as usize);
    let inherited = match input.u8()? {
        0 => false,
        1 => true,
        flag => return Err(Error::Parse(format!("signaling {flag}"))),
    };
    let raw: BitcoinTransaction = deserialize(input.blob()?)
        .map_err(|error| Error::Parse(format!("transaction: {error}")))?;
    let mut prevouts = Vec::with_capacity(raw.input.len());
    for _ in 0..raw.input.len() {
        prevouts.push(match input.u8()? {
            0 => None,
            _ => {
                let value = Sats::from(input.u64()?);
                Some(TxOut::from((
                    bitcoin::ScriptBuf::from_bytes(input.blob()?.to_vec()),
                    value,
                )))
            }
        });
    }
    let txid = Txid::from(raw.compute_txid());
    let wtxid = raw.compute_wtxid();
    let size = raw.total_size();
    let weight = Weight::from(raw.weight());
    let rbf = inherited || raw.input.iter().any(|txin| txin.sequence.is_rbf());
    let mut tx = Transaction {
        index: None,
        txid,
        version: raw.version.into(),
        total_sigop_cost: SigOps::ZERO,
        weight,
        lock_time: raw.lock_time.into(),
        total_size: size,
        fee,
        input: raw
            .input
            .into_iter()
            .zip(prevouts)
            .map(|(txin, prevout)| TxIn {
                is_coinbase: false,
                prevout,
                txid: txin.previous_output.txid.into(),
                vout: Vout::from(txin.previous_output.vout),
                script_sig: txin.script_sig,
                witness: txin.witness,
                sequence: txin.sequence.into(),
            })
            .collect(),
        output: raw.output.into_iter().map(TxOut::from).collect(),
        status: TxStatus::UNCONFIRMED,
    };
    tx.refresh_sigops();
    // Never below what its weight and the sigops its body shows make it.
    let least = tx.total_sigop_cost.adjust_vsize(VSize::from(weight));
    if vsize < u64::from(least) {
        return Err(Error::Parse(format!("vsize {vsize}")));
    }
    let entry = TxEntry {
        txid,
        wtxid,
        fee,
        vsize: VSize::from(vsize),
        weight,
        size: size as u64,
        first_seen,
        rbf,
    };
    Ok(TxRecord::new(Arc::new(tx), entry))
}

/// The graveyard, and the pool a clean shutdown saved with it (none from a periodic save).
fn decode(compressed: &[u8]) -> Result<(TxGraveyard, Vec<TxRecord>)> {
    let claimed = compressed
        .first_chunk::<4>()
        .map(|prefix| u32::from_le_bytes(*prefix) as usize)
        .ok_or_else(|| Error::Parse("truncated".into()))?;
    if claimed > MAX_FILE_BYTES {
        return Err(Error::Parse(format!("claims {claimed} bytes")));
    }
    let bytes = lz4_flex::decompress_size_prepended(compressed)
        .map_err(|error| Error::Parse(format!("lz4: {error}")))?;
    let mut input = Reader(&bytes);
    if input.take(MAGIC.len())? != MAGIC {
        return Err(Error::Parse("not a graveyard of this version".into()));
    }
    // Ages grew by the downtime (none if the clock went back).
    let down = Duration::from_millis(unix_millis().saturating_sub(input.u64()?));
    let now = Instant::now();
    let told = match input.u8()? {
        0 => None,
        _ => {
            let tip = BlockHash::from(bitcoin::BlockHash::from_byte_array(input.array()?));
            // Older than the clock can say: before every tombstone kept.
            let age = Duration::from_micros(input.u64()?);
            Some((tip, now.checked_sub(age + down)))
        }
    };
    let count = input.u64()?;
    let mut graveyard = TxGraveyard::default();
    let mut previous_age = Duration::MAX;
    for _ in 0..count {
        let age = Duration::from_micros(input.u64()?);
        if age > previous_age {
            return Err(Error::Parse("not in burial order".into()));
        }
        previous_age = age;
        let removal = match input.u8()? {
            0 => TxRemoval::Mined,
            1 => TxRemoval::Vanished,
            2 => TxRemoval::Replaced {
                by: Txid::from(bitcoin::Txid::from_byte_array(input.array()?)),
            },
            tag => return Err(Error::Parse(format!("removal {tag}"))),
        };
        let chunk_rate = FeeRate::from_milli(input.u64()?);
        let record = read_record(&mut input)?;
        // Older than the clock can say (past the machine's uptime): nothing of it is kept.
        let Some(removed_at) = now.checked_sub(age + down) else {
            continue;
        };
        graveyard.restore(record, chunk_rate, removal, removed_at);
    }
    let pool = match input.u8()? {
        0 => Vec::new(),
        1 => {
            let count = input.u64()?;
            let mut keys = RapidHashSet::default();
            (0..count)
                .map(|_| {
                    let record = read_record(&mut input)?;
                    if !keys.insert(record.entry.key()) {
                        return Err(Error::Parse("a pool transaction twice".into()));
                    }
                    Ok(record)
                })
                .collect::<Result<_>>()?
        }
        flag => return Err(Error::Parse(format!("pool {flag}"))),
    };
    if !input.0.is_empty() {
        return Err(Error::Parse("trailing bytes".into()));
    }
    if !graveyard.acyclic() {
        return Err(Error::Parse("a replacement cycle".into()));
    }
    if let Some((tip, at)) = told {
        graveyard.restore_told(tip, at);
    }
    Ok((graveyard, pool))
}

#[derive(Default)]
struct Writer(Vec<u8>);

impl Writer {
    fn u8(&mut self, value: u8) {
        self.0.push(value);
    }

    fn u64(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    /// Length-prefixed.
    fn blob(&mut self, bytes: &[u8]) {
        self.u64(bytes.len() as u64);
        self.bytes(bytes);
    }
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.0.len() < len {
            return Err(Error::Parse("truncated".into()));
        }
        let (head, rest) = self.0.split_at(len);
        self.0 = rest;
        Ok(head)
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn array(&mut self) -> Result<[u8; 32]> {
        Ok(self.take(32)?.try_into().unwrap())
    }

    fn blob(&mut self) -> Result<&'a [u8]> {
        let len = usize::try_from(self.u64()?).map_err(|_| Error::Parse("length".into()))?;
        self.take(len)
    }
}
