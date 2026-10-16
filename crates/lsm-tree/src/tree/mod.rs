use std::{
    fs,
    ops::{Bound, Deref, RangeBounds},
    path::Path,
    sync::Arc,
};

use inner::Inner;
use log::{debug, info};
use rustc_hash::FxHashSet;

use crate::{
    Config, Error, InternalValue, RecordBytes, Result, Slice, Table,
    file::{CURRENT_VERSION_FILE, TABLES_FOLDER},
    merge::Merger,
    mvcc_stream::MvccStream,
    range,
    run_reader::RunReader,
    version::{Level, Run, Version, recovery::Recovery},
};

pub mod ingest;
pub mod inner;

/// A tree's table layout pinned for reads: later ingestions and compactions stay invisible to
/// it, and the tables it reads stay on disk while it lives.
///
/// Drop views before reopening the tree: recovery deletes the tables only they pin as orphans
/// and may reuse their ids.
#[derive(Clone)]
pub struct ReadView(Arc<Version>);

impl ReadView {
    /// Iterates over a range of the pinned records using the specified owned record bytes.
    /// Record widths are checked while decoding.
    #[must_use]
    pub fn range_as<K: RecordBytes, V: RecordBytes, B: AsRef<[u8]>, R: RangeBounds<B>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + Send + 'static + use<K, V, B, R> {
        Tree::range_from::<K, V>(&self.0, Tree::owned_bounds(&range))
            .map(|item| item.map(|value| (value.key.user_key, value.value)))
    }

    /// Iterates over the pinned records matching `prefix`.
    #[must_use]
    pub fn prefix_as<K: RecordBytes, V: RecordBytes>(
        &self,
        prefix: &[u8],
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + Send + 'static + use<K, V> {
        self.range_as::<K, V, _, _>(range::prefix_to_range(prefix))
    }
}

/// A table-only log-structured merge tree.
#[derive(Clone)]
pub struct Tree(Arc<Inner>);

impl Deref for Tree {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Tree {
    /// Opens an existing tree or creates an empty one.
    ///
    /// # Errors
    ///
    /// Returns an error when the tree cannot be created, recovered, or read.
    pub fn open(config: Config) -> Result<Self> {
        debug!("Opening LSM tree at {}", config.path.display());

        if config.path.join("version").try_exists()? {
            return Err(Error::InvalidVersion(1));
        }

        if config.path.join(CURRENT_VERSION_FILE).try_exists()? {
            Self::recover(config)
        } else {
            Self::create_new(config)
        }
    }

    /// Starts a strictly sorted direct-to-table ingestion.
    ///
    /// # Errors
    ///
    /// Returns an error when the table writer cannot be created.
    pub fn ingestion(&self) -> Result<ingest::Ingestion<'_>> {
        ingest::Ingestion::new(self)
    }

    /// Starts ingestion using the specified owned record bytes.
    ///
    /// # Errors
    /// Returns an error if the table writer cannot be created.
    pub fn ingestion_as<K: RecordBytes, V: RecordBytes>(
        &self,
    ) -> Result<ingest::Ingestion<'_, K, V>> {
        ingest::Ingestion::new(self)
    }

    /// Reads the latest value for `key`.
    ///
    /// # Errors
    ///
    /// Returns an error when an underlying table cannot be read.
    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Slice>> {
        self.get_as::<Slice>(key.as_ref())
    }

    /// Iterates over all latest key-value pairs.
    #[must_use]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Result<(Slice, Slice)>> + Send + 'static {
        self.range::<&[u8], _>(..)
    }

    /// Iterates over the latest key-value pairs in `range`.
    #[must_use]
    pub fn range<K: AsRef<[u8]>, R: RangeBounds<K>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = Result<(Slice, Slice)>> + Send + 'static {
        self.range_as::<Slice, Slice, K, R>(range)
    }

    /// Iterates over all latest records using the specified owned bytes.
    /// Record widths are checked while decoding.
    #[must_use]
    pub fn iter_as<K: RecordBytes, V: RecordBytes>(
        &self,
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + Send + 'static {
        self.range_as::<K, V, &[u8], _>(..)
    }

    /// Iterates over a range using the specified owned record bytes.
    /// Record widths are checked while decoding.
    #[must_use]
    pub fn range_as<K: RecordBytes, V: RecordBytes, B: AsRef<[u8]>, R: RangeBounds<B>>(
        &self,
        range: R,
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + Send + 'static + use<K, V, B, R> {
        self.read_view().range_as(range)
    }

    /// Pins the current table layout for later reads.
    #[must_use]
    pub fn read_view(&self) -> ReadView {
        ReadView(self.versions.load())
    }

    /// Iterates over the latest key-value pairs matching `prefix`.
    #[must_use]
    pub fn prefix<K: AsRef<[u8]>>(
        &self,
        prefix: K,
    ) -> impl DoubleEndedIterator<Item = Result<(Slice, Slice)>> + Send + 'static {
        self.range(range::prefix_to_range(prefix.as_ref()))
    }

    /// Iterates over matching keys using the specified owned record bytes.
    #[must_use]
    pub fn prefix_as<K: RecordBytes, V: RecordBytes>(
        &self,
        prefix: &[u8],
    ) -> impl DoubleEndedIterator<Item = Result<(K, V)>> + Send + 'static + use<K, V> {
        self.read_view().prefix_as(prefix)
    }

    /// Returns the number of disjoint level-zero runs.
    #[must_use]
    pub fn l0_run_count(&self) -> usize {
        self.versions.guard().level(0).map_or(0, Level::run_count)
    }

    /// Runs leveled compaction until no eligible work remains.
    ///
    /// # Errors
    ///
    /// Returns an error when compaction cannot read, write, or publish its tables.
    pub fn compact(&self) -> Result<()> {
        self.compact_as::<Slice, Slice>()
    }

    /// Compacts a tree whose keys and values use the specified record bytes.
    ///
    /// # Errors
    /// Returns an error if records have a different width, or compaction fails.
    pub fn compact_as<K: RecordBytes, V: RecordBytes>(&self) -> Result<()> {
        self.run_compaction::<K, V>()
    }

    /// Returns the currently published table-layout generation.
    #[doc(hidden)]
    #[must_use]
    pub fn current_version_id(&self) -> u64 {
        self.versions.guard().id()
    }

    /// Reads a value into the specified owned bytes.
    ///
    /// # Errors
    /// Returns an error if a table cannot be read or the value width does not match.
    pub fn get_as<V: RecordBytes>(&self, key: &[u8]) -> Result<Option<V>> {
        let version = self.versions.guard();
        Self::get_from_tables::<V>(&version, key)
    }

    fn get_from_tables<V: RecordBytes>(version: &Version, key: &[u8]) -> Result<Option<V>> {
        let mut key_hash = None;

        for table in version
            .iter_levels()
            .flat_map(Level::iter)
            .filter_map(|run| run.get_for_key(key))
        {
            if let Some(item) = table.get_value(key, &mut key_hash)? {
                return Ok((!item.value_type.is_tombstone()).then_some(item.value));
            }
        }

        Ok(None)
    }

    fn owned_bounds<K: AsRef<[u8]>, R: RangeBounds<K>>(range: &R) -> (Bound<Slice>, Bound<Slice>) {
        let start = match range.start_bound() {
            Bound::Included(key) => Bound::Included(key.as_ref().into()),
            Bound::Excluded(key) => Bound::Excluded(key.as_ref().into()),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match range.end_bound() {
            Bound::Included(key) => Bound::Included(key.as_ref().into()),
            Bound::Excluded(key) => Bound::Excluded(key.as_ref().into()),
            Bound::Unbounded => Bound::Unbounded,
        };
        (start, end)
    }

    fn range_from<K: RecordBytes, V: RecordBytes>(
        version: &Version,
        bounds: (Bound<Slice>, Bound<Slice>),
    ) -> impl DoubleEndedIterator<Item = Result<InternalValue<K, V>>> + Send + 'static + use<K, V>
    {
        let readers = version
            .iter_levels()
            .flat_map(Level::iter)
            .filter_map(|run| RunReader::<K, V>::new(run.clone(), bounds.clone()))
            .collect();

        MvccStream::new(Merger::new(readers)).filter(|item| match item {
            Ok(value) => !value.key.is_tombstone(),
            Err(_) => true,
        })
    }

    fn recover(config: Config) -> Result<Self> {
        info!("Recovering LSM tree at {}", config.path.display());
        let tree_id = Inner::next_tree_id();

        let version = Self::recover_tables(&config.path, tree_id, &config)?;
        Ok(Self(Arc::new(Inner::recover(config, version, tree_id))))
    }

    fn create_new(config: Config) -> Result<Self> {
        let path = config.path.clone();
        fs::create_dir_all(path.join(TABLES_FOLDER))?;

        Ok(Self(Arc::new(Inner::create_new(config)?)))
    }

    /// Opens the manifest's tables into their levels, then deletes table files the manifest
    /// does not list (left by an interrupted ingestion or compaction).
    fn recover_tables(path: &Path, tree_id: u32, config: &Config) -> Result<Version> {
        let recovery = Recovery::load(path)?;
        let tables_path = path.join(TABLES_FOLDER);
        fs::create_dir_all(&tables_path)?;

        let mut live = FxHashSet::default();
        let mut levels = Vec::with_capacity(recovery.table_ids.len());
        for (level, runs) in recovery.table_ids.iter().enumerate() {
            let mut level_runs = Vec::with_capacity(runs.len());
            for run in runs {
                let mut tables = Vec::with_capacity(run.len());
                for table in run {
                    let file = tables_path.join(table.id.to_string());
                    // A listed table without its file cannot be recovered.
                    if !file.try_exists()? {
                        return Err(Error::Unrecoverable);
                    }
                    let recovered =
                        Table::recover(file, table.global_seqno, tree_id, level, config)?;
                    // The file must hold the table the manifest lists under its name.
                    if recovered.id() != table.id {
                        return Err(Error::Unrecoverable);
                    }
                    live.insert(table.id);
                    tables.push(recovered);
                }
                // The manifest loader rejects empty runs.
                level_runs.push(Arc::new(Run::new(tables).ok_or(Error::Unrecoverable)?));
            }
            levels.push(Level::from_runs(level_runs));
        }
        let version = Version::from_levels(recovery.curr_version_id, levels);

        let mut orphaned = Vec::new();
        for entry in fs::read_dir(&tables_path)? {
            let entry = entry?;
            let file_name = entry.file_name();
            if file_name == ".DS_Store" || file_name.to_string_lossy().starts_with("._") {
                continue;
            }

            let table_id = file_name
                .to_str()
                .ok_or(Error::Unrecoverable)?
                .parse::<u32>()
                .map_err(|_| Error::Unrecoverable)?;
            if !live.contains(&table_id) {
                orphaned.push(entry.path());
            }
        }
        for table in orphaned {
            fs::remove_file(table)?;
        }

        Ok(version)
    }
}
