use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use inner::Inner;

use crate::{
    DatabaseBuilder, Error, Keyspace, KeyspaceCreateOptions, Result,
    db_config::Config,
    file::{DATABASE_FORMAT, KEYSPACES_FOLDER, LOCK_FILE, VERSION_MARKER},
    locked_file,
    worker_pool::WorkerPool,
};

mod inner;

/// A BRK database containing named, table-only keyspaces.
#[derive(Clone)]
pub struct Database {
    inner: Arc<Inner>,
}

impl Database {
    /// Creates a database builder rooted at `path`.
    pub fn builder(path: impl AsRef<Path>) -> DatabaseBuilder {
        DatabaseBuilder::new(path.as_ref())
    }

    /// Opens or creates a database.
    #[doc(hidden)]
    pub(crate) fn open(config: Config) -> Result<Self> {
        fs::create_dir_all(&config.path)?;

        let marker_path = config.path.join(VERSION_MARKER);
        let lock = locked_file::lock(&config.path.join(LOCK_FILE))?;
        if marker_path.try_exists()? {
            if fs::read(&marker_path)? != DATABASE_FORMAT {
                return Err(Error::InvalidVersion);
            }
        } else {
            let keyspaces_path = config.path.join(KEYSPACES_FOLDER);
            fs::create_dir_all(&keyspaces_path)?;

            File::create_new(&marker_path)?.write_all(DATABASE_FORMAT)?;
        }

        let worker_pool = WorkerPool::start()?;
        Ok(Self {
            inner: Arc::new(Inner {
                worker_pool,
                keyspaces: Mutex::default(),
                config,
                lock,
            }),
        })
    }

    /// The database's directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.inner.config.path
    }

    /// Waits for every queued or in-flight compaction. Call after all ingestions have joined.
    ///
    /// # Errors
    /// Returns an error if a compaction worker has died.
    #[cfg(feature = "bench-cpu")]
    pub fn wait_for_compaction(&self) -> Result<()> {
        self.inner.worker_pool.wait_idle()
    }

    /// The directory where the named keyspace stores its tables.
    #[must_use]
    pub fn keyspace_path(&self, name: &str) -> PathBuf {
        self.path().join(KEYSPACES_FOLDER).join(name)
    }

    /// The bytes of the published tables of every keyspace open in this process, from memory: no
    /// file is read. The few small files beside them (markers, the lock) are not counted.
    ///
    /// # Panics
    ///
    /// Panics if a keyspace lock is poisoned.
    #[must_use]
    pub fn disk_space(&self) -> u64 {
        self.open_keyspaces().iter().map(Keyspace::disk_space).sum()
    }

    /// Ends the deferral of every keyspace opened deferred
    /// ([`KeyspaceCreateOptions::defer_compaction`]) and queues its merge, once; the others are
    /// left as they are.
    ///
    /// # Panics
    ///
    /// Panics if a keyspace lock is poisoned.
    pub fn resume_compaction(&self) {
        self.open_keyspaces().iter().for_each(Keyspace::resume_compaction);
    }

    /// Whether some keyspace still holds runs that deferral stacked: until their merge after
    /// [`Self::resume_compaction`] is published.
    ///
    /// # Panics
    ///
    /// Panics if a keyspace lock is poisoned.
    #[must_use]
    pub fn is_tiered(&self) -> bool {
        self.open_keyspaces().iter().any(Keyspace::is_tiered)
    }

    /// Every keyspace open in this process.
    fn open_keyspaces(&self) -> Vec<Keyspace> {
        let keyspaces = self
            .inner
            .keyspaces
            .lock()
            .expect("keyspace registry lock is poisoned");
        keyspaces
            .values()
            .filter_map(|slot| slot.lock().expect("keyspace slot lock is poisoned").clone())
            .collect()
    }

    /// Opens or creates `name` using the supplied immutable-table policy.
    ///
    /// The policy closure is evaluated only on the first successful open in this process.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyspace cannot be created or recovered.
    ///
    /// # Panics
    ///
    /// Panics unless `name` is 1 to 255 lowercase ASCII letters, digits, `_` or `-`.
    pub fn keyspace(
        &self,
        name: &str,
        create_options: impl FnOnce() -> KeyspaceCreateOptions,
    ) -> Result<Keyspace> {
        assert!(Self::is_valid_keyspace_name(name), "invalid keyspace name");

        let slot = {
            let mut keyspaces = self
                .inner
                .keyspaces
                .lock()
                .expect("keyspace registry lock is poisoned");
            keyspaces
                .entry(name.to_owned())
                .or_insert_with(|| Arc::new(Mutex::new(None)))
                .clone()
        };
        let mut keyspace = slot.lock().expect("keyspace slot lock is poisoned");
        if let Some(keyspace) = &*keyspace {
            return Ok(keyspace.clone());
        }

        let opened = Keyspace::open(
            name,
            &self.keyspace_path(name),
            create_options(),
            &self.inner.config,
            &self.inner.worker_pool,
            self.inner.lock.clone(),
        )?;
        *keyspace = Some(opened.clone());
        Ok(opened)
    }

    /// Canonical names only: each names its own directory, so no case aliases (case-insensitive
    /// file systems) and no `.` or `..`.
    fn is_valid_keyspace_name(name: &str) -> bool {
        !name.is_empty()
            && u8::try_from(name.len()).is_ok()
            && name.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            })
    }
}
