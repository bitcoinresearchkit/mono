//! One model-based run through the store stack (brk_store, fjall, lsm-tree): three stores, one
//! per kind, share a database and take random batches of the changes the indexer makes, each
//! ingested on its own thread while compaction merges in the background. Every read path is
//! checked against a `BTreeMap`: pending reads, gets, ranges, prefixes, views pinned across later
//! batches and a closed database, and reopens that must drop a stray table.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    fs, mem,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    thread,
};

use brk_error::{Error, Result};
use brk_types::{Sats, StoreValue, TxIndex, Unit, Version};
use fjall::{Database, RecordBytes};

use crate::{AnyStore, Kind, Store, StoreView, open_database};

const VERSION: Version = Version::new(1);
/// Keys' high halves, one prefix each; the scan ends of 0x00FF and 0xFFFF carry (to 0x01, to none).
const HIGH: [u32; 6] = [0x0000, 0x00FF, 0x0100, 0x7FFF, 0xFFFE, 0xFFFF];
/// Low halves under each prefix; the first batch fills about half of them.
const LOW: usize = 1 << 13;
const ROUNDS: usize = 4;
const BATCHES: usize = 50;
/// The most changes in a batch after the first.
const CHANGES: usize = 256;

#[derive(Clone, Copy)]
struct Prefix(u16);
brk_types::impl_store_value!(Prefix, u16);

type Records<V> = BTreeMap<u32, <V as StoreValue>::Bytes>;

#[test]
fn stores_match_model() {
    let path = std::env::temp_dir().join(format!("brk_store-{}", std::process::id()));
    let mut random = Case::new("random", Kind::Random, 1, |n| TxIndex::new(n as u32));
    let mut recent = Case::new("recent", Kind::Recent, 2, Sats::new);
    let mut vec = Case::new("vec", Kind::Vec, 3, |_| Unit);

    for round in 0..ROUNDS {
        let db = open_database(&path).unwrap();
        thread::scope(|scope| {
            let spawn = |name: &str| thread::Builder::new().name(name.to_owned());
            let (db, path) = (&db, &path);
            spawn("random")
                .spawn_scoped(scope, || random.round(db, path, round))
                .unwrap();
            spawn("recent")
                .spawn_scoped(scope, || recent.round(db, path, round))
                .unwrap();
            spawn("vec")
                .spawn_scoped(scope, || vec.round(db, path, round))
                .unwrap();
        });
        drop(db);
        // The views the stores left hold the database lock: no reopen can drop their tables.
        assert!(open_database(&path).is_err_and(|error| error.is_lock_error()));
        random.close(&path);
        recent.close(&path);
        vec.close(&path);
    }

    fs::remove_dir_all(&path).unwrap();
}

/// One store, its model and its readers' views.
struct Case<V: StoreValue> {
    name: &'static str,
    kind: Kind,
    value: fn(u64) -> V,
    rng: Rng,
    ingested: Records<V>,
    /// The batch's changes, `None` a weak delete.
    pending: BTreeMap<u32, Option<V::Bytes>>,
    /// Keys put again over an ingested value, as a BIP30 duplicate is: a weak delete cancels
    /// only the newest value, so the indexer never deletes them.
    overwritten: BTreeSet<u32>,
    /// Pinned views, each with the records it pinned.
    views: Vec<(StoreView<TxIndex, V>, Records<V>)>,
    /// A stray table file the next open must delete.
    orphan: Option<PathBuf>,
    /// Ingestions since the store opened.
    ingestions: usize,
}

impl<V> Case<V>
where
    V: StoreValue + Send + Sync + 'static,
    V::Bytes: RecordBytes,
{
    fn new(name: &'static str, kind: Kind, seed: u64, value: fn(u64) -> V) -> Self {
        Self {
            name,
            kind,
            value,
            rng: Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15)),
            ingested: BTreeMap::new(),
            pending: BTreeMap::new(),
            overwritten: BTreeSet::new(),
            views: Vec::new(),
            orphan: None,
            ingestions: 0,
        }
    }

    /// One open of the store: version checks on the first reopen, then the batches.
    fn round(&mut self, db: &Database, path: &Path, round: usize) {
        let (name, kind) = (self.name, self.kind);
        let import = |version| Store::<TxIndex, V>::import(db, path, name, version, kind);
        if round == 1 {
            // A changed version, or records without one, are refused.
            assert!(matches!(
                import(Version::new(2)),
                Err(Error::VersionMismatch { .. })
            ));
            let marker = path.join("meta").join(name).join("version");
            let saved = fs::read(&marker).unwrap();
            fs::remove_file(&marker).unwrap();
            assert!(matches!(import(VERSION), Err(Error::MissingVersion { .. })));
            fs::write(&marker, saved).unwrap();
        }
        let mut store = import(VERSION).unwrap();
        self.ingestions = 0;
        // The open deleted the stray table (a compaction since may have reused its name).
        if let Some(orphan) = self.orphan.take() {
            assert_ne!(fs::read(orphan).ok().as_deref(), Some(&b"orphan"[..]));
        }
        assert_same(
            &drain(&mut self.rng, store.view().prefix(Unit)),
            &pairs(self.ingested.iter()),
        );

        for _ in 0..BATCHES {
            self.batch(&mut store);
        }
        // A view outlives its store and the database.
        if self.views.is_empty() {
            self.views.push((store.view(), self.ingested.clone()));
        }
    }

    /// Random changes, read through the pending layer, ingested on another thread while a view
    /// is read, then read back every way.
    fn batch(&mut self, store: &mut Store<TxIndex, V>) {
        let changes = if self.ingested.is_empty() {
            HIGH.len() * LOW / 2
        } else {
            1 + self.rng.below(CHANGES)
        };
        for _ in 0..changes {
            self.change(store);
        }
        for _ in 0..64 {
            let key = near(&mut self.rng, &self.pending);
            let want = match self.pending.get(&key) {
                Some(change) => *change,
                None => self.ingested.get(&key).copied(),
            };
            assert_eq!(Self::get(store, key), want);
        }

        let high = HIGH[self.rng.below(HIGH.len())];
        let before = pairs(self.ingested.range(group(high)));
        let touched = self.commit();
        let after = pairs(self.ingested.range(group(high)));
        if let Some(ingest) = store.take_pending_ingest() {
            thread::scope(|scope| {
                let ingesting = scope.spawn(|| ingest.run());
                // Views taken meanwhile hold the batch whole or not at all. Their count varies,
                // so they draw from their own generator.
                let mut rng = Rng(self.rng.next());
                while !ingesting.is_finished() {
                    let seen = drain(&mut rng, store.view().prefix(Prefix(high as u16)));
                    assert!(
                        seen == before || seen == after,
                        "a view mixed two ingestions"
                    );
                }
                ingesting.join().unwrap().unwrap();
            });
            self.ingestions += 1;
        } else {
            assert!(touched.is_empty());
        }

        assert_eq!(store.is_empty().unwrap(), self.ingested.is_empty());
        // The batch's keys, then random ones: most are absent, which filters answer.
        for key in touched.into_iter().chain((0..64).map(|_| self.rng.key())) {
            assert_eq!(Self::get(store, key), self.ingested.get(&key).copied());
        }
        let (mut start, mut end) = (self.rng.key(), self.rng.key());
        if self.rng.coin() {
            end = start.saturating_add(self.rng.below(64) as u32);
        }
        if start > end {
            mem::swap(&mut start, &mut end);
        }
        assert_same(
            &drain(
                &mut self.rng,
                store.range(TxIndex::new(start)..TxIndex::new(end)),
            ),
            &pairs(self.ingested.range(start..end)),
        );

        // Pinned views keep their records through later batches and compactions.
        if self.rng.below(8) == 0 {
            self.views.push((store.view(), self.ingested.clone()));
            if self.views.len() > 3 {
                self.views.remove(0);
            }
        }
        for (view, pinned) in &self.views {
            let high = HIGH[self.rng.below(HIGH.len())];
            assert_same(
                &drain(&mut self.rng, view.prefix(Prefix(high as u16))),
                &pairs(pinned.range(group(high))),
            );
        }
    }

    /// One change the indexer could make in the key's state: a put, a weak delete of a value put
    /// once, a re-put over an ingested value (a BIP30 duplicate), or a batch's put cancelled.
    fn change(&mut self, store: &mut Store<TxIndex, V>) {
        let key = if self.rng.coin() {
            near(&mut self.rng, &self.ingested)
        } else {
            near(&mut self.rng, &self.pending)
        };
        let put = match self.pending.get(&key) {
            // Nothing follows a delete in its batch.
            Some(None) => return,
            // Replace the batch's put, or cancel it (an output spent in its own block).
            Some(Some(_)) => self.rng.coin(),
            None => {
                !self.ingested.contains_key(&key)
                    || self.overwritten.contains(&key)
                    || self.rng.below(16) == 0
            }
        };
        if put {
            let value = (self.value)(self.rng.next());
            store.insert(TxIndex::new(key), value);
            self.pending.insert(key, Some(value.to_store_bytes()));
        } else {
            store.remove(TxIndex::new(key));
            if self.pending.remove(&key).is_none() {
                self.pending.insert(key, None);
            }
        }
    }

    /// Lands the batch in the model and returns its keys.
    fn commit(&mut self) -> Vec<u32> {
        let touched = self.pending.keys().copied().collect();
        for (key, change) in mem::take(&mut self.pending) {
            match change {
                Some(value) => {
                    if self.ingested.insert(key, value).is_some() {
                        self.overwritten.insert(key);
                    }
                }
                None => {
                    self.ingested.remove(&key);
                }
            }
        }
        touched
    }

    /// After the database closed: the views still read what they pinned, then release it, and a
    /// stray table is left where an interrupted compaction leaves one.
    fn close(&mut self, path: &Path) {
        for (view, pinned) in mem::take(&mut self.views) {
            assert_same(
                &drain(&mut self.rng, view.prefix(Unit)),
                &pairs(pinned.iter()),
            );
        }
        let tables = path.join("fjall/keyspaces").join(self.name).join("tables");
        let ids: Vec<u32> = fs::read_dir(&tables)
            .unwrap()
            .map(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .parse()
                    .unwrap()
            })
            .collect();
        // Compaction merged the tables the ingestions wrote.
        assert!(
            ids.len() <= self.ingestions / 2,
            "{}: {} tables",
            self.name,
            ids.len()
        );
        let orphan = tables.join(ids.iter().max().map_or(0, |id| id + 1).to_string());
        fs::write(&orphan, "orphan").unwrap();
        self.orphan = Some(orphan);
    }

    fn get(store: &Store<TxIndex, V>, key: u32) -> Option<V::Bytes> {
        store
            .get(&TxIndex::new(key))
            .unwrap()
            .map(|value| value.to_store_bytes())
    }
}

/// xorshift64*: deterministic and dependency free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn coin(&mut self) -> bool {
        self.next() >> 63 == 1
    }

    fn key(&mut self) -> u32 {
        HIGH[self.below(HIGH.len())] << 16 | self.below(LOW) as u32
    }
}

/// A random key, or the first of `records` from one.
fn near<T>(rng: &mut Rng, records: &BTreeMap<u32, T>) -> u32 {
    let key = rng.key();
    if rng.coin() {
        records.range(key..).next().map_or(key, |(&next, _)| next)
    } else {
        key
    }
}

/// The keys under `high`'s prefix.
fn group(high: u32) -> RangeInclusive<u32> {
    high << 16..=high << 16 | 0xFFFF
}

fn pairs<'a, B: Copy + 'a>(records: impl Iterator<Item = (&'a u32, &'a B)>) -> Vec<(u32, B)> {
    records.map(|(&key, &value)| (key, value)).collect()
}

/// Reads `records` forward, backward, or from both ends at random, as readers page.
fn drain<V: StoreValue>(
    rng: &mut Rng,
    mut records: impl DoubleEndedIterator<Item = Result<(TxIndex, V)>>,
) -> Vec<(u32, V::Bytes)> {
    let order = rng.below(3);
    let (mut front, mut back) = (Vec::new(), Vec::new());
    loop {
        let (record, side) = if order == 1 || (order == 2 && rng.coin()) {
            (records.next_back(), &mut back)
        } else {
            (records.next(), &mut front)
        };
        let Some(record) = record else { break };
        let (key, value) = record.unwrap();
        side.push((*key, value.to_store_bytes()));
    }
    front.extend(back.into_iter().rev());
    front
}

/// Equal record lists, or the first difference.
#[track_caller]
fn assert_same<B: PartialEq + Debug>(got: &[(u32, B)], want: &[(u32, B)]) {
    let len = got.len().max(want.len());
    if let Some(at) = (0..len).find(|&at| got.get(at) != want.get(at)) {
        panic!(
            "record {at} of {}/{}: got {:?}, want {:?}",
            got.len(),
            want.len(),
            got.get(at),
            want.get(at)
        );
    }
}
