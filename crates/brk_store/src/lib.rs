#![doc = include_str!("../README.md")]

use std::{borrow::Cow, fmt::Debug, fs, hash::Hash, mem, ops::Range, path::Path};

use brk_error::Result;
use brk_types::{Height, Version};
use byteview::ByteView;
use fjall::{Database, Keyspace, KeyspaceCreateOptions, PersistMode, config::*};
use rustc_hash::{FxHashMap, FxHashSet};

mod any;
mod item;
mod kind;
mod meta;
mod mode;

pub use any::*;
pub use item::*;
pub use kind::*;
pub use meta::*;
pub use mode::*;

const MAJOR_FJALL_VERSION: Version = Version::new(3);

pub type StoreTask = Box<dyn FnOnce() -> Result<()> + Send>;

pub fn open_database(path: &Path) -> fjall::Result<Database> {
    Database::builder(path.join("fjall"))
        .cache_size(3 * 1024 * 1024 * 1024)
        .max_cached_files(Some(512))
        .open()
}

#[derive(Clone)]
pub struct Store<K, V> {
    database: Database,
    meta: StoreMeta,
    name: &'static str,
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
            database: db.clone(),
            meta,
            name: Box::leak(Box::new(name.to_string())),
            keyspace,
            puts: FxHashMap::default(),
            dels: FxHashSet::default(),
            caches,
        })
    }

    fn open_keyspace(database: &Database, name: &str, _mode: Mode, kind: Kind) -> Result<Keyspace> {
        let mut options = KeyspaceCreateOptions::default()
            .manual_journal_persist(true)
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
                    .max_memtable_size(8 * 1024 * 1024)
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

    /// Takes buffered puts/dels and returns a durable background commit.
    pub fn take_pending_ingest(&mut self, height: Height) -> Result<Option<StoreTask>>
    where
        K: Send + 'static,
        V: Send + 'static,
        for<'a> ByteView: From<&'a K> + From<&'a V>,
    {
        let (ingest, checkpoint) = self.take_pending_ingest_parts(height)?;
        let Some(ingest) = ingest else {
            checkpoint()?;
            return Ok(None);
        };
        let database = self.database.clone();

        Ok(Some(Box::new(move || {
            ingest()?;
            database.persist(PersistMode::SyncData)?;
            checkpoint()
        })))
    }

    /// Takes buffered puts/dels and returns separate ingest and checkpoint tasks.
    ///
    /// The indexer uses this to persist one batch across all stores before
    /// publishing any of their checkpoints.
    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    pub fn take_pending_ingest_parts(
        &mut self,
        height: Height,
    ) -> Result<(Option<StoreTask>, StoreTask)>
    where
        K: Send + 'static,
        V: Send + 'static,
        for<'a> ByteView: From<&'a K> + From<&'a V>,
    {
        let puts = mem::take(&mut self.puts);
        let dels = mem::take(&mut self.dels);

        let meta = self.meta.clone();
        let checkpoint = Box::new(move || {
            if meta.needs(height) {
                meta.export(height)?;
            }
            Ok(())
        });

        let ingest = if puts.is_empty() && dels.is_empty() {
            None
        } else {
            let keyspace = self.keyspace.clone();
            Some(Box::new(move || Self::ingest(&keyspace, puts.iter(), dels.iter())) as StoreTask)
        };

        Ok((ingest, checkpoint))
    }

    /// Ingests buffered writes, persists them, then publishes their height.
    pub fn commit(&mut self, height: Height) -> Result<()>
    where
        for<'a> ByteView: From<&'a K> + From<&'a V>,
    {
        self.ingest_pending()?;
        self.database.persist(PersistMode::SyncData)?;
        self.export_meta_if_needed(height)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (K, V)> {
        self.keyspace
            .iter()
            .map(|res| res.into_inner().unwrap())
            .map(|(k, v)| (K::from(ByteView::from(&*k)), V::from(ByteView::from(&*v))))
    }

    #[inline]
    pub fn prefix<P: Into<ByteView>>(
        &self,
        prefix: P,
    ) -> impl DoubleEndedIterator<Item = (K, V)> + '_ {
        let prefix: ByteView = prefix.into();
        self.keyspace
            .prefix(&*prefix)
            .map(|res| res.into_inner().unwrap())
            .map(|(k, v)| (K::from(ByteView::from(&*k)), V::from(ByteView::from(&*v))))
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
            .map(|res| res.into_inner().unwrap())
            .map(|(k, v)| (K::from(ByteView::from(&*k)), V::from(ByteView::from(&*v))))
    }

    pub fn approximate_len(&self) -> usize {
        self.keyspace.approximate_len()
    }

    #[inline]
    fn has(&self, height: Height) -> bool {
        self.meta.has(height)
    }

    #[inline]
    pub fn needs(&self, height: Height) -> bool {
        self.meta.needs(height)
    }

    fn export_meta(&mut self, height: Height) -> Result<()> {
        self.meta.export(height)?;
        Ok(())
    }

    fn export_meta_if_needed(&mut self, height: Height) -> Result<()> {
        if !self.has(height) {
            self.export_meta(height)?;
        }
        Ok(())
    }

    fn ingest_pending(&mut self) -> Result<()>
    where
        for<'a> ByteView: From<&'a K> + From<&'a V>,
    {
        let puts = mem::take(&mut self.puts);
        let dels = mem::take(&mut self.dels);

        if puts.is_empty() && dels.is_empty() {
            return Ok(());
        }

        Self::ingest(&self.keyspace, puts.iter(), dels.iter())?;

        if !self.caches.is_empty() {
            self.caches.pop();
            self.caches.insert(0, puts);
        }

        Ok(())
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
    fn keyspace(&self) -> &Keyspace {
        &self.keyspace
    }

    fn export_meta(&mut self, height: Height) -> Result<()> {
        self.export_meta(height)
    }

    fn export_meta_sync(&mut self, height: Height) -> Result<()> {
        self.meta.export_sync(height)?;
        Ok(())
    }

    fn export_meta_if_needed(&mut self, height: Height) -> Result<()> {
        self.export_meta_if_needed(height)
    }

    fn ingest_pending(&mut self) -> Result<()> {
        self.ingest_pending()
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn height(&self) -> Option<Height> {
        self.meta.height()
    }

    fn has(&self, height: Height) -> bool {
        self.has(height)
    }

    fn needs(&self, height: Height) -> bool {
        self.needs(height)
    }

    fn version(&self) -> Version {
        self.meta.version()
    }

    fn commit(&mut self, height: Height) -> Result<()> {
        self.commit(height)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process,
    };

    use brk_types::Unit;

    use super::*;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("brk-store-{name}-{}", process::id()));
            let _ = fs::remove_dir_all(&path);
            Self(path)
        }
    }

    impl AsRef<Path> for TestDir {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn import_test_store(db: &Database, path: &Path) -> Result<Store<Height, Unit>> {
        Store::import(db, path, "test", Version::ZERO, Mode::Any, Kind::Random)
    }

    #[test]
    fn dropped_pending_ingest_does_not_advance_checkpoint() -> Result<()> {
        let path = TestDir::new("dropped-ingest");
        let height = Height::new(42);
        let key = Height::new(7);

        let db = open_database(path.as_ref())?;
        let mut store = import_test_store(&db, path.as_ref())?;
        store.insert(key, Unit);

        let ingest = store.take_pending_ingest(height)?;
        assert!(store.needs(height));
        drop(ingest);
        drop(store);
        drop(db);

        let db = open_database(path.as_ref())?;
        let store = import_test_store(&db, path.as_ref())?;
        assert!(store.needs(height));
        assert!(store.get(&key)?.is_none());

        Ok(())
    }

    #[test]
    fn successful_pending_ingest_advances_checkpoint() -> Result<()> {
        let path = TestDir::new("successful-ingest");
        let height = Height::new(42);
        let key = Height::new(7);

        let db = open_database(path.as_ref())?;
        let mut store = import_test_store(&db, path.as_ref())?;
        store.insert(key, Unit);

        let ingest = store.take_pending_ingest(height)?;
        ingest.unwrap()()?;

        assert!(!store.needs(height));
        assert!(store.get(&key)?.is_some());
        drop(store);
        drop(db);

        let db = open_database(path.as_ref())?;
        let store = import_test_store(&db, path.as_ref())?;
        assert!(!store.needs(height));
        assert!(store.get(&key)?.is_some());

        Ok(())
    }

    #[test]
    fn failed_checkpoint_write_keeps_old_height() -> Result<()> {
        let path = TestDir::new("failed-checkpoint");
        let height = Height::new(42);
        let key = Height::new(7);

        let db = open_database(path.as_ref())?;
        let mut store = import_test_store(&db, path.as_ref())?;
        store.insert(key, Unit);

        let (ingest, checkpoint) = store.take_pending_ingest_parts(height)?;
        ingest.unwrap()()?;
        db.persist(PersistMode::SyncData)?;
        fs::remove_dir_all(path.as_ref().join("meta/test"))?;

        assert!(checkpoint().is_err());
        assert!(store.needs(height));
        assert!(store.get(&key)?.is_some());

        Ok(())
    }
}
