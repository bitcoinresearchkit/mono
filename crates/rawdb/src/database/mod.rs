mod background_tasks;
mod data_file;
mod hole_punch;
mod inner;
mod layout;
mod locked_file;
mod metadata_file;
mod mmap;
mod owner;
mod regions;
pub(crate) mod weak;

use std::{
    fmt,
    fs::{self, File},
    path::Path,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use log::debug;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{Error, PAGE_SIZE, Region, RegionMetadata, Result};

use self::{
    background_tasks::BackgroundTasks, data_file::DataFile, inner::DatabaseInner, layout::Layout,
    owner::DatabaseOwner, regions::Regions,
};

/// Memory-mapped database with region-based storage and hole punching.
#[derive(Clone)]
#[must_use = "Database should be stored to keep the database open"]
pub struct Database {
    pub(crate) inner: Arc<DatabaseOwner>,
}

impl Database {
    /// Opens or creates a database at `path`.
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        let data = DataFile::open(&path.join("data"))?;
        let regions = Regions::open(path)?;

        let owner = Arc::new_cyclic(|foreground| {
            let storage = Arc::new(DatabaseInner {
                path: path.to_owned(),
                foreground: foreground.clone(),
                writes: RwLock::new(()),
                layout: RwLock::new(Layout::default()),
                regions: RwLock::new(regions),
                data,
                tasks: BackgroundTasks::default(),
            });
            let background = Arc::new(DatabaseOwner {
                storage: storage.clone(),
                background: None,
            });
            DatabaseOwner {
                storage,
                background: Some(background),
            }
        });
        let db = Self { inner: owner };

        let imported = db.regions_mut().fill(&db)?;
        *db.layout_mut() = Layout::try_from(imported)?;

        debug!("{}: opened with {} regions", db, db.regions().len());

        Ok(db)
    }

    /// Cached mapped file length (no syscall).
    pub(crate) fn file_len(&self) -> usize {
        self.inner.data.len()
    }

    /// Grows the file if needed (doubles size, 1 MiB floor, sparse-file friendly).
    pub fn set_min_len(&self, len: usize) -> Result<()> {
        if len <= self.file_len() {
            return Ok(());
        }
        self.inner.data.ensure_len(len, &self.inner.writes)
    }

    /// Identifiers of every region in the database, sorted.
    pub fn region_ids(&self) -> Vec<String> {
        let mut ids = self.regions().ids().map(str::to_owned).collect::<Vec<_>>();
        ids.sort_unstable();
        ids
    }

    pub fn get_region(&self, id: &str) -> Option<Region> {
        let region = self.regions().get(id).cloned();
        if let Some(region) = &region {
            region.0.mark_accessed();
        }
        region
    }

    pub fn create_region_if_needed(&self, id: &str) -> Result<Region> {
        if let Some(region) = self.get_region(id) {
            return Ok(region);
        }
        RegionMetadata::validate_id(id)?;
        loop {
            let writes = self.inner.writes.read();
            let mut layout = self.layout_mut();
            let mut regions = self.regions_mut();
            if let Some(region) = regions.get(id).cloned() {
                region.0.mark_accessed();
                return Ok(region);
            }
            let hole = layout.find_smallest_adequate_hole(PAGE_SIZE);
            let start = hole.unwrap_or_else(|| layout.end());
            let end = start
                .checked_add(PAGE_SIZE)
                .ok_or(Error::FileSizeOverflow { requested: start })?;
            if end > self.file_len() {
                // Readers can still inspect the registry or flush while a
                // remap waits for them. Recheck allocation after growing.
                drop(regions);
                drop(layout);
                drop(writes);
                self.inner.data.ensure_len(end, &self.inner.writes)?;
                continue;
            }
            let region = regions.create(self, Arc::from(id), start)?;
            if hole.is_some() {
                layout.consume_hole(start, PAGE_SIZE);
                region.0.tail_needs_punch.store(true, Ordering::Relaxed);
            }
            layout.insert_region(start, &region);
            region.0.mark_accessed();
            return Ok(region);
        }
    }

    pub fn remove_region_if_exists(&self, id: &str) -> Result<()> {
        match self.remove_region(id) {
            Ok(()) | Err(Error::RegionNotFound) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn remove_region(&self, id: &str) -> Result<()> {
        let Some(region) = self.get_region(id) else {
            return Err(Error::RegionNotFound);
        };
        region.remove()
    }

    /// Removes every region that has not been returned by [`Self::get_region`]
    /// or [`Self::create_region_if_needed`] since this database was opened.
    pub fn retain_accessed_regions(&self) -> Result<()> {
        let layout = self.inner.layout.read();
        let regions_to_remove: Vec<_> = layout
            .regions()
            .filter(|region| !region.0.was_accessed())
            .cloned()
            .collect();
        drop(layout);
        debug!(
            "{}: removing {} unused regions",
            self,
            regions_to_remove.len()
        );
        for region in regions_to_remove {
            region.remove()?;
        }
        let _writes = self.inner.writes.write();
        self.regions_mut().shrink_to_fit()?;
        self.flush_inner(&_writes);
        Ok(())
    }

    /// Opens the data file read-only (for external consumers like mmap readers).
    pub(crate) fn open_read_only_file(&self) -> Result<File> {
        File::open(self.path().join("data")).map_err(Error::from)
    }

    /// Makes freed space reusable. Writes go straight to the shared mapping, so they are visible to
    /// the next open (soft quits included) without syncing; power loss is out of scope.
    pub fn flush(&self) {
        let _writes = self.inner.writes.write();
        self.flush_inner(&_writes);
    }

    fn flush_inner(&self, _writes: &RwLockWriteGuard<'_, ()>) {
        // Pending holes become reusable once their metadata is written.
        self.layout_mut().promote_pending_holes();
    }

    /// Cancellable wait for use inside `run_bg` closures. Returns
    /// immediately when `sync_bg_tasks` is called.
    pub fn bg_sleep(&self, dur: Duration) {
        self.inner.tasks.sleep(dur);
    }

    /// Waits five seconds before compacting, off the caller's save path.
    /// Intended for background tasks; `sync_bg_tasks` cuts the wait short.
    pub fn compact_deferred_default(&self) -> Result<()> {
        self.bg_sleep(Duration::from_secs(5));
        self.compact()
    }

    /// Flushes, then punches holes to reclaim disk space.
    pub fn compact(&self) -> Result<()> {
        let _writes = self.inner.writes.write();
        let i = Instant::now();
        self.flush_inner(&_writes);
        let r = self.punch_holes(&_writes);
        debug!("{}: compact in {:?}", self, i.elapsed());
        r
    }

    /// Runs `f` on a background thread. The last foreground owner joins pending
    /// tasks on drop. Use `sync_bg_tasks()` to observe errors explicitly.
    /// Background callbacks must not join their own database's tasks.
    pub fn run_bg(&self, f: impl FnOnce(&Self) -> Result<()> + Send + 'static) {
        // The worker owns the storage, but not the foreground shutdown guard.
        let db = Self {
            inner: self
                .inner
                .background
                .as_ref()
                .unwrap_or(&self.inner)
                .clone(),
        };
        self.inner.tasks.spawn(move || f(&db));
    }

    /// Wakes `bg_sleep` waiters and joins every pending task, even if one fails
    /// or panics. Returns the first error after all tasks have been joined.
    pub fn sync_bg_tasks(&self) -> Result<()> {
        self.inner.tasks.join()
    }

    fn punch_holes(&self, _writes: &RwLockWriteGuard<'_, ()>) -> Result<()> {
        let mut layout = self.layout_mut();
        let mut punched = 0usize;

        // Keep each region boundary stable while deriving and punching its tail.
        for region in layout.regions() {
            if !region.0.tail_needs_punch.load(Ordering::Relaxed) {
                continue;
            }

            // SAFETY: compaction holds the exclusive mutation barrier.
            let (rstart, len, reserved) = unsafe { region.0.bounds() };
            let ceil_len = len.next_multiple_of(PAGE_SIZE);

            if ceil_len < reserved {
                let start = rstart + ceil_len;
                let hole = reserved - ceil_len;
                self.inner.data.punch_hole(start, hole)?;
                punched += 1;
            }

            region.0.tail_needs_punch.store(false, Ordering::Relaxed);
        }

        if layout.holes_need_punch() {
            // The layout write lock prevents holes from being allocated while
            // they are punched. No per-region lock is needed for free space.
            for (&start, &hole) in layout.start_to_hole() {
                self.inner.data.punch_hole(start, hole)?;
            }
            punched += layout.start_to_hole().len();
            // Removed trailing regions leave no metadata after reopening. Reclaim
            // the unused file tail too, including any old allocated pages there.
            let end = self.file_len() / PAGE_SIZE * PAGE_SIZE;
            let start = layout.end();
            if start < end {
                self.inner.data.punch_hole(start, end - start)?;
                punched += 1;
            }
            layout.mark_holes_punched();
        }

        drop(layout);

        // KEEP_SIZE preserves file length; the kernel zeroes punched pages.
        debug!("{}: punched {} holes", self, punched);
        Ok(())
    }

    pub(crate) fn regions(&self) -> RwLockReadGuard<'_, Regions> {
        self.inner.regions.read()
    }

    pub(crate) fn regions_mut(&self) -> RwLockWriteGuard<'_, Regions> {
        self.inner.regions.write()
    }

    pub(crate) fn layout_mut(&self) -> RwLockWriteGuard<'_, Layout> {
        self.inner.layout.write()
    }

    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    #[inline]
    fn name(&self) -> &str {
        self.path()
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
