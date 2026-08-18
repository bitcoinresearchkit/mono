#![doc = include_str!("../README.md")]

use std::{borrow::Cow, cmp::Ordering, fmt::Debug, fs, hash::Hash, mem, ops::Range, path::Path};

use brk_error::Result;
use brk_types::Version;
use byteview::ByteView;
use fjall::{Database, Keyspace, KeyspaceCreateOptions, config::*};
use rustc_hash::{FxHashMap, FxHashSet};

mod any;
mod item;
mod kind;
mod meta;
mod mode;

use item::Item;
use meta::StoreMeta;

pub use any::*;
pub use kind::*;
pub use mode::*;

const MAJOR_FJALL_VERSION: Version = Version::new(6);

pub fn open_database(path: &Path) -> fjall::Result<Database> {
    Database::builder(path.join("fjall"))
        .cache_size(3 * 1024 * 1024 * 1024)
        .max_cached_files(512)
        .open()
}

pub type PendingIngest = Box<dyn FnOnce() -> Result<()> + Send>;

#[derive(Clone)]
pub struct Store<K, V> {
    meta: StoreMeta,
    keyspace: Keyspace,
    puts: FxHashMap<K, V>,
    dels: FxHashSet<K>,
    caches: Vec<FxHashMap<K, V>>,
}

impl<K, V> Store<K, V>
where
    K: Debug + Clone + From<ByteView> + Ord + Eq + Hash,
    V: Debug + Clone + From<ByteView>,
    ByteView: From<K> + From<V>,
    Self: Send + Sync,
{
    pub fn import(
        db: &Database,
        path: &Path,
        name: &str,
        version: Version,
        mode: Mode,
        kind: Kind,
    ) -> Result<Self> {
        Self::import_inner(db, path, name, version, mode, kind, 0)
    }

    pub fn import_cached(
        db: &Database,
        path: &Path,
        name: &str,
        version: Version,
        mode: Mode,
        kind: Kind,
        max_batches: u8,
    ) -> Result<Self> {
        Self::import_inner(db, path, name, version, mode, kind, max_batches)
    }

    fn import_inner(
        db: &Database,
        path: &Path,
        name: &str,
        version: Version,
        mode: Mode,
        kind: Kind,
        max_batches: u8,
    ) -> Result<Self> {
        fs::create_dir_all(path)?;

        let (meta, keyspace) = StoreMeta::checked_open(
            &path.join(format!("meta/{name}")),
            MAJOR_FJALL_VERSION + version,
            || {
                Self::open_keyspace(db, name, mode, kind).inspect_err(|e| {
                    eprintln!("{e}");
                    eprintln!("Delete {path:?} and try again");
                })
            },
        )?;

        let mut caches = vec![];
        for _ in 0..max_batches {
            caches.push(FxHashMap::default());
        }

        Ok(Self {
            meta,
            keyspace,
            puts: FxHashMap::default(),
            dels: FxHashSet::default(),
            caches,
        })
    }

    fn open_keyspace(database: &Database, name: &str, _mode: Mode, kind: Kind) -> Result<Keyspace> {
        let mut options = KeyspaceCreateOptions::default()
            .filter_block_partitioning_policy(PartitioningPolicy::new([false, false, true]))
            .index_block_partitioning_policy(PartitioningPolicy::new([false, false, true]));

        match kind {
            Kind::Random => {
                options = options
                    .filter_block_pinning_policy(PinningPolicy::new([true, true, true, false]))
                    .filter_policy(FilterPolicy::new([
                        FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(
                            0.0001,
                        )),
                        FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.001)),
                        FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)),
                        FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(9.0)),
                    ]));
            }
            Kind::Recent => {
                options = options
                    .expect_point_read_hits(true)
                    .filter_policy(FilterPolicy::new([
                        FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(
                            0.0001,
                        )),
                        FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.001)),
                        FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(8.0)),
                        FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(7.0)),
                    ]));
            }
            Kind::Vec => {
                options = options
                    .data_block_restart_interval_policy(RestartIntervalPolicy::all(8))
                    .filter_policy(FilterPolicy::disabled())
                    .filter_block_pinning_policy(PinningPolicy::all(false))
                    .index_block_pinning_policy(PinningPolicy::all(false));
            }
        }

        database.keyspace(name, || options).map_err(|e| e.into())
    }

    #[inline]
    pub fn get<'a>(&'a self, key: &'a K) -> Result<Option<Cow<'a, V>>>
    where
        ByteView: From<&'a K>,
    {
        if let Some(v) = self.puts.get(key) {
            return Ok(Some(Cow::Borrowed(v)));
        }

        for cache in &self.caches {
            if let Some(v) = cache.get(key) {
                return Ok(Some(Cow::Borrowed(v)));
            }
        }

        if let Some(slice) = self.keyspace.get(ByteView::from(key))? {
            Ok(Some(Cow::Owned(V::from(ByteView::from(slice)))))
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub fn is_empty(&self) -> Result<bool> {
        self.keyspace.is_empty().map_err(|e| e.into())
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        let _ = self.dels.is_empty() || self.dels.remove(&key);
        self.puts.insert(key, value);
    }

    #[inline]
    pub fn remove(&mut self, key: K) {
        if self.puts.remove(&key).is_some() {
            return;
        }
        let newly_inserted = self.dels.insert(key);
        debug_assert!(newly_inserted, "Double deletion at {:?}", self.meta.path());
    }

    /// Clear all caches. Call after bulk removals (e.g., rollback) to prevent stale reads.
    #[inline]
    pub fn clear_caches(&mut self) {
        for cache in &mut self.caches {
            *cache = FxHashMap::default();
        }
    }

    /// Takes buffered puts/dels and returns a closure that ingests them into the keyspace.
    /// The store is left with empty buffers, ready for the next batch. The caller must
    /// persist the database after ingestion before treating the data as durable.
    pub fn take_pending_ingest(&mut self) -> Option<PendingIngest>
    where
        K: Send + 'static,
        V: Send + 'static,
        for<'a> ByteView: From<&'a K> + From<&'a V>,
    {
        let puts = mem::take(&mut self.puts);
        let dels = mem::take(&mut self.dels);

        if puts.is_empty() && dels.is_empty() {
            return None;
        }

        let keyspace = self.keyspace.clone();

        Some(Box::new(move || Self::ingest_owned(&keyspace, puts, dels)))
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (K, V)> {
        self.keyspace
            .iter()
            .map(Result::unwrap)
            .map(|(k, v)| (K::from(ByteView::from(k)), V::from(ByteView::from(v))))
    }

    #[inline]
    pub fn prefix<P: Into<ByteView>>(
        &self,
        prefix: P,
    ) -> impl DoubleEndedIterator<Item = (K, V)> + '_ {
        let prefix: ByteView = prefix.into();
        self.keyspace
            .prefix(prefix)
            .map(Result::unwrap)
            .map(|(k, v)| (K::from(ByteView::from(k)), V::from(ByteView::from(v))))
    }

    #[inline]
    pub fn range<B: Into<ByteView>>(
        &self,
        range: Range<B>,
    ) -> impl DoubleEndedIterator<Item = (K, V)> + '_ {
        let start: ByteView = range.start.into();
        let end: ByteView = range.end.into();
        self.keyspace
            .range(start..end)
            .map(Result::unwrap)
            .map(|(k, v)| (K::from(ByteView::from(k)), V::from(ByteView::from(v))))
    }

    pub fn approximate_len(&self) -> usize {
        self.keyspace.approximate_len()
    }

    fn ingest<'a>(
        keyspace: &Keyspace,
        puts: impl Iterator<Item = (&'a K, &'a V)>,
        dels: impl Iterator<Item = &'a K>,
    ) -> Result<()>
    where
        ByteView: From<&'a K> + From<&'a V>,
        K: 'a,
        V: 'a,
    {
        let mut items: Vec<Item<&'a K, &'a V>> = puts
            .map(|(key, value)| Item::Value { key, value })
            .chain(dels.map(Item::Tomb))
            .collect();

        items.sort_unstable();

        let mut ingestion = keyspace.start_ingestion()?;
        // FxHashMap/FxHashSet keep keys unique and disjoint; sorting therefore
        // proves the strict ordering required by ingestion.
        for item in items {
            match item {
                Item::Value { key, value } => {
                    ingestion.write(ByteView::from(key), ByteView::from(value))?;
                }
                Item::Tomb(key) => {
                    ingestion.write_weak_tombstone(ByteView::from(key))?;
                }
            }
        }
        // Store keyspaces are mutated only through these ingestion phases, so
        // no journaled Fjall write can race their completion.
        ingestion.finish()?;

        Ok(())
    }

    fn ingest_owned(keyspace: &Keyspace, puts: FxHashMap<K, V>, dels: FxHashSet<K>) -> Result<()> {
        let mut puts: Vec<_> = puts.into_iter().collect();
        let mut dels: Vec<_> = dels.into_iter().collect();

        puts.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        dels.sort_unstable();

        let mut puts = puts.into_iter().peekable();
        let mut dels = dels.into_iter().peekable();
        let mut ingestion = keyspace.start_ingestion()?;

        // The buffers are unique and disjoint, and this merge emits them in
        // strict key order, so release builds can skip re-cloning each key.
        while puts.peek().is_some() || dels.peek().is_some() {
            match (puts.peek(), dels.peek()) {
                (Some((put_key, _)), Some(del_key)) => match put_key.cmp(del_key) {
                    Ordering::Less => {
                        let (key, value) = puts.next().unwrap();
                        ingestion.write(ByteView::from(key), ByteView::from(value))?;
                    }
                    Ordering::Greater => {
                        ingestion.write_weak_tombstone(ByteView::from(dels.next().unwrap()))?;
                    }
                    Ordering::Equal => unreachable!("key is both inserted and deleted"),
                },
                (Some(_), None) => {
                    let (key, value) = puts.next().unwrap();
                    ingestion.write(ByteView::from(key), ByteView::from(value))?;
                }
                (None, Some(_)) => {
                    ingestion.write_weak_tombstone(ByteView::from(dels.next().unwrap()))?;
                }
                (None, None) => break,
            }
        }

        // Store keyspaces are mutated only through these ingestion phases, so
        // no journaled Fjall write can race their completion.
        ingestion.finish()?;
        Ok(())
    }
}

impl<K, V> AnyStore for Store<K, V>
where
    K: Debug + Clone + From<ByteView> + Ord + Eq + Hash,
    V: Debug + Clone + From<ByteView>,
    for<'a> ByteView: From<K> + From<V> + From<&'a K> + From<&'a V>,
    Self: Send + Sync,
{
    fn ingest_pending(&mut self) -> Result<()> {
        let puts = mem::take(&mut self.puts);
        let dels = mem::take(&mut self.dels);

        if puts.is_empty() && dels.is_empty() {
            return Ok(());
        }

        if self.caches.is_empty() {
            Self::ingest_owned(&self.keyspace, puts, dels)?;
        } else {
            Self::ingest(&self.keyspace, puts.iter(), dels.iter())?;
            self.caches.pop();
            self.caches.insert(0, puts);
        }

        Ok(())
    }
}
