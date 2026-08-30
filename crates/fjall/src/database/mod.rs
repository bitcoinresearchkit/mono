mod inner;

use crate::{
    DatabaseBuilder, Keyspace, KeyspaceCreateOptions,
    db_config::Config,
    file::{DATABASE_FORMAT, KEYSPACES_FOLDER, LOCK_FILE, VERSION_MARKER},
    locked_file::LockedFileGuard,
    worker_pool::WorkerPool,
};
use inner::Inner;
use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

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
    pub fn open(config: Config) -> crate::Result<Self> {
        std::fs::create_dir_all(&config.path)?;

        let marker_path = config.path.join(VERSION_MARKER);
        let lock = LockedFileGuard::create_new(&config.path.join(LOCK_FILE))?;
        if marker_path.try_exists()? {
            if std::fs::read(&marker_path)? != DATABASE_FORMAT {
                return Err(crate::Error::InvalidVersion);
            }
        } else {
            let keyspaces_path = config.path.join(KEYSPACES_FOLDER);
            std::fs::create_dir_all(&keyspaces_path)?;

            let mut marker = File::create_new(&marker_path)?;
            marker.write_all(DATABASE_FORMAT)?;
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

    /// Opens or creates `name` using the supplied immutable-table policy.
    ///
    /// The policy closure is evaluated only on the first successful open in this process.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyspace cannot be created or recovered.
    pub fn keyspace(
        &self,
        name: &str,
        create_options: impl FnOnce() -> KeyspaceCreateOptions,
    ) -> crate::Result<Keyspace> {
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
            create_options(),
            &self.inner.config,
            &self.inner.worker_pool,
            self.inner.lock.clone(),
        )?;
        *keyspace = Some(opened.clone());
        Ok(opened)
    }

    fn is_valid_keyspace_name(name: &str) -> bool {
        !name.is_empty()
            && u8::try_from(name.len()).is_ok()
            && name.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'#' | b'$')
            })
    }
}
