pub mod ingest;
pub mod inner;

use crate::{
    BoxedIterator, Config, Error, InternalValue, Result, Slice, Table,
    file::{CURRENT_VERSION_FILE, TABLES_FOLDER},
    merge::Merger,
    mvcc_stream::MvccStream,
    run_reader::RunReader,
    version::{Version, recovery::Recovery},
};
use inner::Inner;
use std::{
    fs,
    ops::{Bound, Deref, RangeBounds},
    path::Path,
    sync::Arc,
};

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
        log::debug!("Opening LSM tree at {}", config.path.display());

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

    /// Reads the latest value for `key`.
    ///
    /// # Errors
    ///
    /// Returns an error when an underlying table cannot be read.
    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<Slice>> {
        let version = self.versions.guard();
        Self::get_from_tables(&version, key.as_ref())
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
        let bounds = Self::owned_bounds(&range);
        let version = self.versions.load();

        Self::range_from(&version, bounds)
            .map(|item| item.map(|value| (value.key.user_key, value.value)))
    }

    /// Iterates over the latest key-value pairs matching `prefix`.
    #[must_use]
    pub fn prefix<K: AsRef<[u8]>>(
        &self,
        prefix: K,
    ) -> impl DoubleEndedIterator<Item = Result<(Slice, Slice)>> + Send + 'static {
        self.range(crate::range::prefix_to_range(prefix.as_ref()))
    }

    /// Returns the number of disjoint level-zero runs.
    #[must_use]
    pub fn l0_run_count(&self) -> usize {
        self.versions
            .guard()
            .level(0)
            .map_or(0, crate::version::Level::run_count)
    }

    /// Runs leveled compaction until no eligible work remains.
    ///
    /// # Errors
    ///
    /// Returns an error when compaction cannot read, write, or publish its tables.
    pub fn compact(&self) -> Result<()> {
        crate::compaction::worker::Worker::new(self).run()
    }

    /// Returns the currently published table-layout generation.
    #[doc(hidden)]
    #[must_use]
    pub fn current_version_id(&self) -> u64 {
        self.versions.guard().id()
    }

    fn get_from_tables(version: &Version, key: &[u8]) -> Result<Option<Slice>> {
        let mut key_hash = None;

        for table in version
            .iter_levels()
            .flat_map(crate::version::Level::iter)
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

    fn range_from(
        version: &Version,
        bounds: (Bound<Slice>, Bound<Slice>),
    ) -> impl DoubleEndedIterator<Item = Result<InternalValue>> + Send + 'static + use<> {
        let mut readers: Vec<BoxedIterator<'static>> = Vec::new();
        let overlap = (
            bounds.0.as_ref().map(AsRef::as_ref),
            bounds.1.as_ref().map(AsRef::as_ref),
        );

        for run in version.iter_levels().flat_map(crate::version::Level::iter) {
            match run.len() {
                0 => {}
                1 => {
                    if let Some(table) = run.first()
                        && table.check_key_range_overlap(&overlap)
                    {
                        readers.push(BoxedIterator::new(table.range(bounds.clone())));
                    }
                }
                _ => {
                    if let Some(reader) = RunReader::new(run.clone(), bounds.clone()) {
                        readers.push(BoxedIterator::new(reader));
                    }
                }
            }
        }

        MvccStream::new(Merger::new(readers)).filter(|item| match item {
            Ok(value) => !value.key.is_tombstone(),
            Err(_) => true,
        })
    }

    fn recover(config: Config) -> Result<Self> {
        log::info!("Recovering LSM tree at {}", config.path.display());
        let tree_id = Inner::next_tree_id();

        let version = Self::recover_tables(&config.path, tree_id, &config)?;
        Ok(Self(Arc::new(Inner::recover(config, version, tree_id))))
    }

    fn create_new(config: Config) -> Result<Self> {
        let path = config.path.clone();
        fs::create_dir_all(path.join(TABLES_FOLDER))?;

        Ok(Self(Arc::new(Inner::create_new(config)?)))
    }

    fn recover_tables(path: &Path, tree_id: u32, config: &Config) -> Result<Version> {
        let recovery = Recovery::load(path)?;
        let mut expected: rustc_hash::FxHashMap<u32, (u8, u64)> = rustc_hash::FxHashMap::default();

        for (level_index, runs) in (0_u8..).zip(&recovery.table_ids) {
            for table in runs.iter().flatten() {
                expected.insert(table.id, (level_index, table.global_seqno));
            }
        }

        let tables_path = path.join(crate::file::TABLES_FOLDER);
        fs::create_dir_all(&tables_path)?;
        let mut tables = Vec::with_capacity(expected.len());
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

            if let Some(&(level, global_seqno)) = expected.get(&table_id) {
                tables.push(Table::recover(
                    entry.path(),
                    global_seqno,
                    tree_id,
                    config.cache.clone(),
                    config.descriptor_table.clone(),
                    config.filter_block_pinning_policy.get(level.into()),
                    config.index_block_pinning_policy.get(level.into()),
                )?);
            } else {
                orphaned.push(entry.path());
            }
        }

        if tables.len() != expected.len() {
            return Err(Error::Unrecoverable);
        }

        let version = Version::from_recovery(&recovery, &tables)?;
        for table in orphaned {
            fs::remove_file(table)?;
        }

        Ok(version)
    }
}
