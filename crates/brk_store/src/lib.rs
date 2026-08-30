#![doc = include_str!("../README.md")]

use std::{borrow::Cow, cmp::Ordering, fmt::Debug, fs, hash::Hash, ops::Range, path::Path};

use brk_error::Result;
use brk_types::Version;
use byteview::ByteView;
use fjall::{Database, Keyspace, KeyspaceCreateOptions, config::*};
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

const MAJOR_FJALL_VERSION: Version = Version::new(7);
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
        kind: Kind,
    ) -> Result<Self> {
        fs::create_dir_all(path)?;

        let keyspace = checked_open(
            &path.join(format!("meta/{name}")),
            MAJOR_FJALL_VERSION + version,
            || {
                Self::open_keyspace(db, name, kind).inspect_err(|e| {
                    eprintln!("{e}");
                    eprintln!("Delete {path:?} and try again");
                })
            },
        )?;
        Ok(Self {
            keyspace,
            pending: Pending::new(kind),
        })
    }

    fn open_keyspace(database: &Database, name: &str, kind: Kind) -> Result<Keyspace> {
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
        if let Some(pending) = self.pending.get(key) {
            return Ok(pending.map(Cow::Borrowed));
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
        self.pending.insert(key, value);
    }

    #[inline]
    pub fn remove(&mut self, key: K) {
        self.pending.remove(key);
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
        let pending = self.pending.take();

        if pending.is_empty() {
            return None;
        }

        let keyspace = self.keyspace.clone();

        Some(PendingIngest::new(move || {
            Self::ingest_owned(&keyspace, pending)
        }))
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (K, V)> {
        self.keyspace
            .iter()
            .map(|result| result.unwrap())
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
            .map(|result| result.unwrap())
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
            .map(|result| result.unwrap())
            .map(|(k, v)| (K::from(ByteView::from(k)), V::from(ByteView::from(v))))
    }

    fn ingest_owned(keyspace: &Keyspace, pending: Pending<K, V>) -> Result<()>
    where
        for<'a> ByteView: From<&'a K> + From<&'a V>,
    {
        match pending {
            Pending::Hashed { puts, dels } => Self::ingest_hashed(keyspace, puts, dels),
            Pending::Sequential(changes) => Self::ingest_sequential(keyspace, changes),
        }
    }

    fn ingest_hashed(keyspace: &Keyspace, puts: FxHashMap<K, V>, dels: FxHashSet<K>) -> Result<()> {
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

    fn ingest_sequential(keyspace: &Keyspace, mut changes: Vec<Item<K, V>>) -> Result<()>
    where
        for<'a> ByteView: From<&'a K> + From<&'a V>,
    {
        // Equal-key operations must retain their arrival order.
        changes.sort_by(|left, right| left.key().cmp(right.key()));

        let mut changes = changes.into_iter().peekable();
        let mut pending = None;
        let mut ingestion = keyspace.start_ingestion()?;
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
                        ingestion.write(ByteView::from(key), ByteView::from(value))?;
                    }
                    Item::Tomb(key) => {
                        ingestion.write_weak_tombstone(ByteView::from(key))?;
                    }
                }
            }
        }

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
        let pending = self.pending.take();

        if pending.is_empty() {
            return Ok(());
        }

        Self::ingest_owned(&self.keyspace, pending)?;

        Ok(())
    }
}
