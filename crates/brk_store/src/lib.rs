#![doc = include_str!("../README.md")]

use std::{
    hash::Hash,
    iter,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    path::Path,
};

use brk_error::{Error, Result};
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
mod bytes;
#[cfg(test)]
mod tests;

use item::Item;
use meta::checked_open;
use pending::Pending;

pub use any::*;
pub use kind::*;
pub use pending_ingest::{PendingIngest, Ready};
pub use bytes::ByteStore;

const BLOCK_CACHE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub fn open_database(path: &Path) -> Result<Database> {
    let path = path.join("fjall");
    Database::builder(&path)
        .cache_size(BLOCK_CACHE_BYTES)
        .max_cached_files(512)
        .open()
        .map_err(|error| Error::from(error).opening_storage(&path))
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

        // Random and Recent serve point reads of hashed keys, which LZ4 barely shrinks:
        // decompressing every block they read was about a fifth of the indexer's lookup CPU.
        match kind {
            Kind::Random => {
                options = options
                    .data_block_compression(CompressionType::None)
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
                    .data_block_compression(CompressionType::None)
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

        database.keyspace(name, || options).map_err(|error| {
            Error::from(error).opening_storage(&database.keyspace_path(name))
        })
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
    pub fn insert(&mut self, key: K, value: V) {
        self.pending.insert(key, value);
    }

    #[inline]
    pub fn remove(&mut self, key: K) {
        self.pending.remove(key);
    }

    /// Pins the ingested records for later reads. Call it with nothing pending: the view holds
    /// only ingested records.
    pub fn view(&self) -> StoreView<K, V> {
        debug_assert!(
            self.pending.is_empty(),
            "a store view excludes pending writes"
        );
        StoreView {
            view: self.keyspace.read_view(),
            records: PhantomData,
        }
    }

    /// The ingested records in `range`, storage errors as `Err` items.
    #[inline]
    pub fn range<B: StoreValue>(
        &self,
        range: impl RangeBounds<B>,
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + '_ {
        let bytes = |bound: Bound<&B>| bound.map(StoreValue::to_store_bytes);
        self.keyspace
            .range_as::<K::Bytes, V::Bytes, _, _>((
                bytes(range.start_bound()),
                bytes(range.end_bound()),
            ))
            .map(|result| {
                let (k, v) = result?;
                Ok((K::from_store_bytes(k), V::from_store_bytes(v)))
            })
    }

    /// The record under `key` as ingested, without the pending writes.
    fn ingested(keyspace: &Keyspace, key: &K) -> Result<Option<V::Bytes>> {
        Ok(keyspace.get_as::<V::Bytes>(key.to_store_bytes().as_ref())?)
    }

    /// Whether ingesting `change` changes what the keyspace holds.
    fn changes(keyspace: &Keyspace, change: &Item<K, V>) -> Result<bool> {
        Ok(match change {
            Item::Value { key, value } => Self::ingested(keyspace, key)?
                .is_none_or(|stored| stored.as_ref() != value.to_store_bytes().as_ref()),
            Item::Tomb(key) => Self::ingested(keyspace, key)?.is_some(),
        })
    }

    fn ingest_hashed(
        keyspace: &Keyspace,
        puts: FxHashMap<K, V>,
        dels: FxHashSet<K>,
        ready: Ready<'_>,
        changes_only: bool,
    ) -> Result<()> {
        let mut puts: Vec<_> = puts.into_iter().collect();
        let mut dels: Vec<_> = dels.into_iter().collect();

        puts.sort_unstable_by_key(|(key, _)| *key);
        dels.sort_unstable();

        let mut puts = puts.into_iter().peekable();
        let mut dels = dels.into_iter().peekable();
        // The buffers are unique and disjoint, and this merge emits them in strict key order.
        let changes = iter::from_fn(|| match (puts.peek(), dels.peek()) {
            (Some((put_key, _)), Some(del_key)) if put_key > del_key => dels.next().map(Item::Tomb),
            (Some((put_key, _)), Some(del_key)) if put_key == del_key => {
                unreachable!("key is both inserted and deleted")
            }
            (Some(_), _) => puts.next().map(|(key, value)| Item::Value { key, value }),
            (None, _) => dels.next().map(Item::Tomb),
        });
        Self::ingest(keyspace, changes, ready, changes_only)
    }

    fn ingest_sequential(
        keyspace: &Keyspace,
        mut changes: Vec<Item<K, V>>,
        ready: Ready<'_>,
        changes_only: bool,
    ) -> Result<()> {
        // Equal-key operations must retain their arrival order.
        changes.sort_by(|left, right| left.key().cmp(right.key()));
        let changes = changes
            .chunk_by(|left, right| left.key() == right.key())
            .filter_map(|same_key| Item::reduce(same_key).cloned());
        Self::ingest(keyspace, changes, ready, changes_only)
    }

    /// Writes `changes`, in strict key order, into one table published once `ready` returns
    /// `Ok`; with `changes_only`, only those that change what the keyspace holds.
    fn ingest(
        keyspace: &Keyspace,
        changes: impl Iterator<Item = Item<K, V>>,
        ready: Ready<'_>,
        changes_only: bool,
    ) -> Result<()> {
        let mut ingestion = keyspace.start_ingestion_as::<K::Bytes, V::Bytes>()?;
        for change in changes {
            if changes_only && !Self::changes(keyspace, &change)? {
                continue;
            }
            match change {
                Item::Value { key, value } => {
                    ingestion.write(key.to_store_bytes(), value.to_store_bytes())?;
                }
                Item::Tomb(key) => ingestion.write_weak_tombstone(key.to_store_bytes())?,
            }
        }

        // Store keyspaces are mutated only through these ingestion phases, so
        // no journaled Fjall write can race their completion.
        ready()?;
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
    K: StoreValue + Ord + Hash + Send + 'static,
    V: StoreValue + Send + 'static,
    K::Bytes: RecordBytes,
    V::Bytes: RecordBytes,
    Self: Send + Sync,
{
    fn is_empty(&self) -> Result<bool> {
        self.keyspace.is_empty().map_err(|e| e.into())
    }

    fn take_pending_ingest(&mut self) -> Option<PendingIngest> {
        let pending = self.pending.take();

        if pending.is_empty() {
            return None;
        }

        let keyspace = self.keyspace.clone();

        Some(PendingIngest::new(move |ready, changes_only| match pending {
            Pending::Hashed { puts, dels } => {
                Self::ingest_hashed(&keyspace, puts, dels, ready, changes_only)
            }
            Pending::Sequential(changes) => {
                Self::ingest_sequential(&keyspace, changes, ready, changes_only)
            }
        }))
    }
}
