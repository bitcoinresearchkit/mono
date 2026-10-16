#![doc = include_str!("../README.md")]

use std::{cmp::Ordering, fs, hash::Hash, marker::PhantomData, ops::Range, path::Path};

use brk_error::Result;
use brk_types::{StoreValue, Version};
use fjall::{
    Database, FORMAT_VERSION, Keyspace, KeyspaceCreateOptions, ReadView, RecordBytes, config::*,
};
use rustc_hash::{FxHashMap, FxHashSet};

mod any;
mod item;
mod kind;
mod meta;
mod pending;
mod pending_ingest;

use item::Item;
use meta::checked_open;
use pending::Pending;

pub use any::*;
pub use kind::*;
pub use pending_ingest::PendingIngest;

const BLOCK_CACHE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub fn open_database(path: &Path) -> Result<Database> {
    Ok(Database::builder(path.join("fjall"))
        .cache_size(BLOCK_CACHE_BYTES)
        .max_cached_files(512)
        .open()?)
}

#[derive(Clone)]
pub struct Store<K, V> {
    keyspace: Keyspace,
    pending: Pending<K, V>,
}

impl<K, V> Store<K, V>
where
    K: StoreValue + Ord + Hash,
    V: StoreValue,
    K::Bytes: RecordBytes,
    V::Bytes: RecordBytes,
    Self: Send + Sync,
{
    pub fn import(
        db: &Database,
        path: &Path,
        name: &str,
        version: Version,
        kind: Kind,
    ) -> Result<Self> {
        fs::create_dir_all(path)?;

        let keyspace = checked_open(
            &path.join(format!("meta/{name}")),
            Version::new(u32::from(FORMAT_VERSION)) + version,
            || Self::open_keyspace(db, name, kind),
        )?;
        Ok(Self {
            keyspace,
            pending: Pending::new(kind),
        })
    }

    fn open_keyspace(database: &Database, name: &str, kind: Kind) -> Result<Keyspace> {
        let mut options = KeyspaceCreateOptions::default()
            .compaction_records::<K::Bytes, V::Bytes>()
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
    pub fn get(&self, key: &K) -> Result<Option<V>> {
        if let Some(pending) = self.pending.get(key) {
            return Ok(pending.copied());
        }
        Ok(self
            .keyspace
            .get_as::<V::Bytes>(key.to_store_bytes().as_ref())?
            .map(V::from_store_bytes))
    }

    #[inline]
    pub fn is_empty(&self) -> Result<bool> {
        self.keyspace.is_empty().map_err(|e| e.into())
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        self.pending.insert(key, value);
    }

    #[inline]
    pub fn remove(&mut self, key: K) {
        self.pending.remove(key);
    }

    /// Takes buffered puts/dels and returns a closure that ingests them into the keyspace.
    /// The store is left with empty buffers, ready for the next batch. A successful ingestion
    /// persists and publishes the records: it writes immutable tables and its manifest.
    pub fn take_pending_ingest(&mut self) -> Option<PendingIngest>
    where
        K: Send + 'static,
        V: Send + 'static,
    {
        let pending = self.pending.take();

        if pending.is_empty() {
            return None;
        }

        let keyspace = self.keyspace.clone();

        Some(PendingIngest::new(move || {
            Self::ingest_owned(&keyspace, pending)
        }))
    }

    /// Pins the ingested records for later reads. Call it with nothing pending: the view holds
    /// only ingested records.
    pub fn view(&self) -> StoreView<K, V> {
        debug_assert!(
            self.pending.is_empty(),
            "a store view excludes pending writes"
        );
        self.ingested()
    }

    fn ingested(&self) -> StoreView<K, V> {
        StoreView {
            view: self.keyspace.read_view(),
            records: PhantomData,
        }
    }

    /// The ingested records in `range`, storage errors as `Err` items.
    #[inline]
    pub fn range<B: StoreValue>(
        &self,
        range: Range<B>,
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + '_ {
        let start = range.start.to_store_bytes();
        let end = range.end.to_store_bytes();
        self.keyspace
            .range_as::<K::Bytes, V::Bytes, _, _>(start..end)
            .map(|result| {
                let (k, v) = result?;
                Ok((K::from_store_bytes(k), V::from_store_bytes(v)))
            })
    }

    fn ingest_owned(keyspace: &Keyspace, pending: Pending<K, V>) -> Result<()> {
        match pending {
            Pending::Hashed { puts, dels } => Self::ingest_hashed(keyspace, puts, dels),
            Pending::Sequential(changes) => Self::ingest_sequential(keyspace, changes),
        }
    }

    fn ingest_hashed(keyspace: &Keyspace, puts: FxHashMap<K, V>, dels: FxHashSet<K>) -> Result<()> {
        let mut puts: Vec<_> = puts.into_iter().collect();
        let mut dels: Vec<_> = dels.into_iter().collect();

        puts.sort_unstable_by_key(|(key, _)| *key);
        dels.sort_unstable();

        let mut puts = puts.into_iter().peekable();
        let mut dels = dels.into_iter().peekable();
        let mut ingestion = keyspace.start_ingestion_as::<K::Bytes, V::Bytes>()?;

        // The buffers are unique and disjoint, and this merge emits them in
        // strict key order, so release builds can skip re-cloning each key.
        while puts.peek().is_some() || dels.peek().is_some() {
            match (puts.peek(), dels.peek()) {
                (Some((put_key, _)), Some(del_key)) => match put_key.cmp(del_key) {
                    Ordering::Less => {
                        let (key, value) = puts.next().unwrap();
                        ingestion.write(key.to_store_bytes(), value.to_store_bytes())?;
                    }
                    Ordering::Greater => {
                        ingestion.write_weak_tombstone(dels.next().unwrap().to_store_bytes())?;
                    }
                    Ordering::Equal => unreachable!("key is both inserted and deleted"),
                },
                (Some(_), None) => {
                    let (key, value) = puts.next().unwrap();
                    ingestion.write(key.to_store_bytes(), value.to_store_bytes())?;
                }
                (None, Some(_)) => {
                    ingestion.write_weak_tombstone(dels.next().unwrap().to_store_bytes())?;
                }
                (None, None) => break,
            }
        }

        // Store keyspaces are mutated only through these ingestion phases, so
        // no journaled Fjall write can race their completion.
        ingestion.finish()?;
        Ok(())
    }

    fn ingest_sequential(keyspace: &Keyspace, mut changes: Vec<Item<K, V>>) -> Result<()> {
        // Equal-key operations must retain their arrival order.
        changes.sort_by(|left, right| left.key().cmp(right.key()));

        let mut changes = changes.into_iter().peekable();
        let mut pending = None;
        let mut ingestion = keyspace.start_ingestion_as::<K::Bytes, V::Bytes>()?;
        while let Some(change) = changes.next() {
            let same_key_follows = changes
                .peek()
                .is_some_and(|next| next.key() == change.key());
            change.apply_to(&mut pending);

            if same_key_follows {
                continue;
            }

            if let Some(pending) = pending.take() {
                match pending {
                    Item::Value { key, value } => {
                        ingestion.write(key.to_store_bytes(), value.to_store_bytes())?;
                    }
                    Item::Tomb(key) => {
                        ingestion.write_weak_tombstone(key.to_store_bytes())?;
                    }
                }
            }
        }

        ingestion.finish()?;
        Ok(())
    }
}

/// A store's ingested records pinned at one moment: later writes and ingestions stay invisible
/// to it, and the tables it reads stay on disk while it lives.
pub struct StoreView<K, V> {
    view: ReadView,
    records: PhantomData<fn() -> (K, V)>,
}

impl<K, V> Clone for StoreView<K, V> {
    fn clone(&self) -> Self {
        Self {
            view: self.view.clone(),
            records: PhantomData,
        }
    }
}

impl<K, V> StoreView<K, V>
where
    K: StoreValue,
    V: StoreValue,
    K::Bytes: RecordBytes,
    V::Bytes: RecordBytes,
{
    /// The pinned records under `prefix`, storage errors as `Err` items.
    #[inline]
    pub fn prefix<P: StoreValue>(
        &self,
        prefix: P,
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + use<K, V, P> {
        self.view
            .prefix_as::<K::Bytes, V::Bytes>(prefix.to_store_bytes().as_ref())
            .map(|result| {
                let (k, v) = result?;
                Ok((K::from_store_bytes(k), V::from_store_bytes(v)))
            })
    }
}

impl<K, V> AnyStore for Store<K, V>
where
    K: StoreValue + Ord + Hash,
    V: StoreValue,
    K::Bytes: RecordBytes,
    V::Bytes: RecordBytes,
    Self: Send + Sync,
{
    fn ingest_pending(&mut self) -> Result<()> {
        let pending = self.pending.take();

        if pending.is_empty() {
            return Ok(());
        }

        Self::ingest_owned(&self.keyspace, pending)?;

        Ok(())
    }
}
