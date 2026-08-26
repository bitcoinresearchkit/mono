#![doc = include_str!("../README.md")]

use std::{
    collections::HashSet,
    fmt,
    fs::{self, File, OpenOptions},
    mem::ManuallyDrop,
    path::{Path, PathBuf},
    sync::{
        Arc, Weak,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use log::{debug, trace};
use memmap2::MmapMut;
use parking_lot::{Condvar, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

mod disk_usage;
pub mod error;
mod hints;
mod hole_punch;
mod layout;
mod mmap;
mod reader;
mod region;
mod region_metadata;
mod regions;

pub use disk_usage::*;
pub use error::*;
pub use hints::*;
use hole_punch::*;
use layout::*;
use mmap::*;
pub use reader::*;
pub use region::*;
pub use region_metadata::*;
use regions::*;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_MINUS_1: usize = PAGE_SIZE - 1;
/// One gibibyte (1024^3 bytes).
#[allow(non_upper_case_globals)]
pub const GiB: usize = 1024 * 1024 * 1024;

/// Memory-mapped database with region-based storage and hole punching.
#[derive(Clone)]
#[must_use = "Database should be stored to keep the database open"]
pub struct Database(Arc<DatabaseInner>);

/// Lock ordering: layout → regions → mmap → file → meta → dirty_ranges.
struct DatabaseInner {
    path: PathBuf,
    name: String,
    layout: RwLock<Layout>,
    regions: RwLock<Regions>,
    mmap: RwLock<MmapMut>,
    file: RwLock<File>,
    cached_file_len: AtomicUsize,
    bg_tasks: Mutex<Vec<JoinHandle<crate::Result<()>>>>,
    bg_sync: (Mutex<bool>, Condvar),
}

impl Database {
    /// Opens or creates a database at `path`.
    pub fn open(path: &Path) -> crate::Result<Self> {
        Self::open_with_min_len(path, 0)
    }

    pub fn open_with_min_len(path: &Path, min_len: usize) -> crate::Result<Self> {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        fs::create_dir_all(path)?;

        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .truncate(false)
            .open(Self::data_path_from(path))?;

        file.try_lock()?;

        let mut file_len = file.metadata()?.len() as usize;
        if file_len < min_len {
            file.set_len(min_len as u64)?;
            file.sync_all()?;
            file_len = min_len;
        }

        let regions = Regions::open(path)?;
        let mmap = create_mmap(&file)?;

        let db = Self(Arc::new(DatabaseInner {
            path: path.to_owned(),
            name,
            layout: RwLock::new(Layout::default()),
            regions: RwLock::new(regions),
            mmap: RwLock::new(mmap),
            file: RwLock::new(file),
            cached_file_len: AtomicUsize::new(file_len),
            bg_tasks: Mutex::new(Vec::new()),
            bg_sync: (Mutex::new(false), Condvar::new()),
        }));

        db.regions_mut().fill(&db)?;
        *db.layout_mut() = Layout::from(&*db.regions());

        debug!("{}: opened with {} regions", db, db.regions().len());

        Ok(db)
    }

    /// Cached file length (no syscall).
    #[inline]
    pub fn file_len(&self) -> usize {
        self.0.cached_file_len.load(Ordering::Relaxed)
    }

    /// Grows the file if needed (doubles size, 1 MiB floor, sparse-file friendly).
    pub fn set_min_len(&self, len: usize) -> crate::Result<()> {
        let len = Self::ceil_number_to_page_size_multiple(len);

        if self.file_len() >= len {
            return Ok(());
        }

        trace!("{}: set_min_len acquiring mmap_mut", self);
        let mut mmap = self.mmap_mut();
        trace!("{}: set_min_len acquiring file_mut", self);
        let file = self.file_mut();

        // Re-check after acquiring lock (another thread may have grown the file).
        let current_len = self.file_len();
        if current_len >= len {
            return Ok(());
        }

        let target_len =
            Self::ceil_number_to_page_size_multiple(len.max(current_len * 2).max(1024 * 1024));
        debug!(
            "{}: set_min_len to {} (requested {})",
            self, target_len, len
        );
        file.set_len(target_len as u64)?;
        self.0.cached_file_len.store(target_len, Ordering::Relaxed);
        *mmap = create_mmap(&file)?;
        Ok(())
    }

    pub fn get_region(&self, id: &str) -> Option<Region> {
        self.regions().get_from_id(id).cloned()
    }

    pub fn create_region_if_needed(&self, id: &str) -> crate::Result<Region> {
        if let Some(region) = self.get_region(id) {
            return Ok(region);
        }

        let layout = self.layout();
        if layout.find_smallest_adequate_hole(PAGE_SIZE).is_none() {
            let end = layout.len();
            drop(layout);
            self.set_min_len(end + PAGE_SIZE)?;
        } else {
            drop(layout);
        }

        debug!("{}: create_region_if_needed '{}'", self, id);
        trace!(
            "{}: create_region_if_needed '{}' acquiring layout_mut",
            self, id
        );
        let mut layout = self.layout_mut();
        trace!(
            "{}: create_region_if_needed '{}' acquiring regions_mut",
            self, id
        );
        let mut regions = self.regions_mut();

        if let Some(region) = regions.get_from_id(id).cloned() {
            return Ok(region);
        }

        let (start, reused_hole) =
            if let Some(start) = layout.find_smallest_adequate_hole(PAGE_SIZE) {
                layout.remove_or_compress_hole(start, PAGE_SIZE)?;
                (start, true)
            } else {
                (layout.len(), false)
            };

        let region = regions.create(self, id.to_owned(), start)?;
        if reused_hole {
            region.meta_mut().mark_tail_needs_punch();
        }
        layout.insert_region(start, &region);
        Ok(region)
    }

    #[inline]
    pub(crate) fn write(&self, start: usize, data: &[u8]) {
        write_to_mmap(&self.mmap(), start, data);
    }

    pub(crate) fn copy(&self, src: usize, dst: usize, len: usize) -> crate::Result<()> {
        if len == 0 {
            return Ok(());
        }

        let src_end = src + len;
        let dst_end = dst + len;
        if !(src_end <= dst || dst_end <= src) {
            return Err(Error::OverlappingCopyRanges {
                src,
                src_end,
                dst,
                dst_end,
            });
        }

        let mmap = self.mmap();
        write_to_mmap(&mmap, dst, &mmap[src..src_end]);
        Ok(())
    }

    pub fn remove_region_if_exists(&self, id: &str) -> crate::Result<()> {
        match self.remove_region(id) {
            Ok(()) | Err(Error::RegionNotFound) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn remove_region(&self, id: &str) -> crate::Result<()> {
        let Some(region) = self.get_region(id) else {
            return Err(Error::RegionNotFound);
        };
        region.remove()
    }

    /// Removes all regions except those in `ids`.
    pub fn retain_regions(&self, mut ids: HashSet<String>) -> crate::Result<()> {
        debug!(
            "{}: retain_regions called with {} ids to keep",
            self,
            ids.len()
        );

        let regions = self.regions();
        let regions_to_remove: Vec<_> = regions
            .id_to_index()
            .keys()
            .filter(|id| !ids.remove(&**id))
            .filter_map(|id| regions.get_from_id(id).cloned())
            .collect();
        drop(regions);

        if !ids.is_empty() {
            debug!(
                "{}: retain_regions: {} ids in retain set not found in db: {:?}",
                self,
                ids.len(),
                ids
            );
        }

        if !regions_to_remove.is_empty() {
            debug!(
                "{}: retain_regions removing {} regions: {:?}",
                self,
                regions_to_remove.len(),
                regions_to_remove
                    .iter()
                    .map(|r| r.meta().id().to_string())
                    .collect::<Vec<_>>()
            );
        }

        for region in regions_to_remove {
            let ref_count = Arc::strong_count(region.arc());
            debug!(
                "{}: removing '{}' (arc count: {})",
                self,
                region.meta().id(),
                ref_count
            );
            region.remove()?;
        }
        self.regions_mut().shrink_to_fit()?;
        Ok(())
    }

    /// Opens the data file read-only (for external consumers like mmap readers).
    #[inline]
    pub fn open_read_only_file(&self) -> crate::Result<File> {
        File::open(self.data_path()).map_err(Error::from)
    }

    pub fn disk_usage(&self) -> crate::Result<DiskUsage> {
        DiskUsage::from_file(&self.file())
    }

    /// Flushes all dirty data and metadata to disk.
    /// Returns the number of regions whose data was flushed.
    pub fn flush(&self) -> crate::Result<usize> {
        let dirty_regions: Vec<(Region, Vec<(usize, usize)>)> = self
            .regions()
            .index_to_region()
            .iter()
            .flatten()
            .filter_map(|r| {
                let ranges = r.take_dirty_ranges();
                if !ranges.is_empty() {
                    Some((r.clone(), ranges))
                } else {
                    None
                }
            })
            .collect();

        let mut flush_ranges = dirty_regions
            .iter()
            .flat_map(|(region, ranges)| {
                let region_start = region.meta().start();
                ranges
                    .iter()
                    .map(move |&(start, end)| (region_start + start, region_start + end))
            })
            .collect::<Vec<_>>();

        flush_ranges.sort_unstable();
        let mut merged_ranges: Vec<(usize, usize)> = Vec::with_capacity(flush_ranges.len());
        for (start, end) in flush_ranges {
            if let Some((_, previous_end)) = merged_ranges.last_mut()
                && start <= *previous_end
            {
                *previous_end = (*previous_end).max(end);
            } else {
                merged_ranges.push((start, end));
            }
        }

        if !merged_ranges.is_empty() {
            let mmap = self.mmap();
            for &(start, end) in &merged_ranges {
                if let Err(error) = mmap.flush_async_range(start, end - start) {
                    drop(mmap);
                    for (region, ranges) in &dirty_regions {
                        region.restore_dirty_ranges(ranges);
                    }
                    return Err(error.into());
                }
            }

            if let Err(error) = self.file().sync_data() {
                for (region, ranges) in &dirty_regions {
                    region.restore_dirty_ranges(ranges);
                }
                return Err(error.into());
            }
        }

        // Data must be durable before metadata can expose it. Holding layout
        // prevents a new pending hole from appearing between metadata sync and
        // promotion.
        let mut layout = self.layout_mut();
        let metadata_flushed = self.regions().flush()?;
        layout.promote_pending_holes(self.name());
        if dirty_regions.is_empty() && !metadata_flushed {
            debug!("{}: flush (no dirty)", self);
            return Ok(0);
        }
        debug!(
            "{}: flushed {} data regions (metadata: {})",
            self,
            dirty_regions.len(),
            metadata_flushed
        );
        Ok(dirty_regions.len())
    }

    /// Gives the OS time to write dirty mmap pages before fsyncing.
    /// Intended for background tasks where the delay is invisible.
    /// Cancellable: `sync_bg_tasks` cuts the wait short.
    pub fn compact_deferred(&self, delay: Duration) -> crate::Result<()> {
        self.bg_sleep(delay);
        self.compact()
    }

    /// Cancellable wait for use inside `run_bg` closures. Returns
    /// immediately when `sync_bg_tasks` is called.
    pub fn bg_sleep(&self, dur: Duration) {
        let (m, cv) = &self.0.bg_sync;
        let mut g = m.lock();
        if !*g {
            cv.wait_for(&mut g, dur);
        }
    }

    /// Like `compact_deferred` with a 5-second default delay.
    pub fn compact_deferred_default(&self) -> crate::Result<()> {
        self.compact_deferred(Duration::from_secs(5))
    }

    /// Flushes, then punches holes to reclaim disk space.
    #[inline]
    pub fn compact(&self) -> crate::Result<()> {
        let i = Instant::now();
        self.flush()?;
        let flush_time = i.elapsed();
        let i = Instant::now();
        let r = self.punch_holes();
        let punch_time = i.elapsed();
        debug!(
            "{}: compact in {:?} (flush: {:?}, punch_holes: {:?})",
            self,
            flush_time + punch_time,
            flush_time,
            punch_time
        );
        r
    }

    /// Runs `f` on a background thread without incrementing the Arc refcount,
    /// so `strong_count` reflects only real owners.
    /// Call `sync_bg_tasks()` before the next write to this database.
    pub fn run_bg(&self, f: impl FnOnce(&Self) -> crate::Result<()> + Send + 'static) {
        // Safety: sync_bg_tasks (called explicitly or from Drop at strong_count == 1)
        // joins this thread before the Arc is deallocated.
        // ManuallyDrop prevents the refcount decrement we never incremented.
        let db = ManuallyDrop::new(unsafe { Self(Arc::from_raw(Arc::as_ptr(&self.0))) });
        self.0.bg_tasks.lock().push(thread::spawn(move || f(&db)));
    }

    /// Wakes any `bg_sleep` waiters and joins all pending background tasks.
    pub fn sync_bg_tasks(&self) -> crate::Result<()> {
        {
            let (m, cv) = &self.0.bg_sync;
            *m.lock() = true;
            cv.notify_all();
        }
        let handles: Vec<_> = self.0.bg_tasks.lock().drain(..).collect();
        for handle in handles {
            handle.join().unwrap()?;
        }
        *self.0.bg_sync.0.lock() = false;
        Ok(())
    }

    fn punch_holes(&self) -> crate::Result<()> {
        let mut layout = self.layout_mut();

        let regions_to_check: Vec<Region> = {
            let regions = self.regions();
            regions
                .index_to_region()
                .iter()
                .flatten()
                .cloned()
                .collect()
        };

        let file = self.file();
        let mut punched = 0usize;

        // Keep each region boundary stable while deriving and punching its tail.
        for region in &regions_to_check {
            let mut meta = region.meta_mut();
            if !meta.tail_needs_punch() {
                continue;
            }

            let rstart = meta.start();
            let len = meta.len();
            let reserved = meta.reserved();
            let ceil_len = Self::ceil_number_to_page_size_multiple(len);

            if ceil_len < reserved {
                let start = rstart + ceil_len;
                let hole = reserved - ceil_len;
                HolePunch::punch(&file, start, hole)?;
                punched += 1;
            }

            meta.mark_tail_punched();
        }

        if layout.holes_need_punch() {
            // The layout write lock prevents holes from being allocated while
            // they are punched. No per-region lock is needed for free space.
            for (&start, &hole) in layout.start_to_hole() {
                HolePunch::punch(&file, start, hole)?;
            }
            punched += layout.start_to_hole().len();
            layout.mark_holes_punched();
        }

        drop(file);
        drop(layout);

        // No mmap recreation needed: KEEP_SIZE preserves file length, kernel zeroes punched pages.
        if punched > 0 {
            debug!("{}: punch_holes syncing after {} punches", self, punched);
            let file = self.file();
            file.sync_data()?;
        }

        Ok(())
    }

    #[inline(always)]
    pub fn file(&self) -> RwLockReadGuard<'_, File> {
        self.0.file.read()
    }

    #[inline(always)]
    pub fn file_mut(&self) -> RwLockWriteGuard<'_, File> {
        self.0.file.write()
    }

    #[inline(always)]
    pub fn mmap(&self) -> RwLockReadGuard<'_, MmapMut> {
        self.0.mmap.read()
    }

    #[inline(always)]
    pub fn mmap_mut(&self) -> RwLockWriteGuard<'_, MmapMut> {
        self.0.mmap.write()
    }

    #[inline(always)]
    pub fn regions(&self) -> RwLockReadGuard<'_, Regions> {
        self.0.regions.read()
    }

    #[inline(always)]
    pub(crate) fn regions_mut(&self) -> RwLockWriteGuard<'_, Regions> {
        self.0.regions.write()
    }

    #[inline(always)]
    pub fn layout(&self) -> RwLockReadGuard<'_, Layout> {
        self.0.layout.read()
    }

    #[inline(always)]
    pub(crate) fn layout_mut(&self) -> RwLockWriteGuard<'_, Layout> {
        self.0.layout.write()
    }

    #[inline]
    fn ceil_number_to_page_size_multiple(num: usize) -> usize {
        (num + PAGE_SIZE_MINUS_1) & !PAGE_SIZE_MINUS_1
    }

    #[inline(always)]
    fn data_path(&self) -> PathBuf {
        Self::data_path_from(self.path())
    }
    #[inline(always)]
    fn data_path_from(path: &Path) -> PathBuf {
        path.join("data")
    }

    #[inline(always)]
    pub fn path(&self) -> &Path {
        &self.0.path
    }

    #[inline]
    pub fn weak_clone(&self) -> WeakDatabase {
        WeakDatabase(Arc::downgrade(&self.0))
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.0.name
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if Arc::strong_count(&self.0) == 1 {
            let _ = self.sync_bg_tasks();
        }
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Weak reference to a [`Database`], held by regions to avoid reference cycles.
#[derive(Debug, Clone)]
pub struct WeakDatabase(Weak<DatabaseInner>);

impl WeakDatabase {
    pub fn upgrade(&self) -> Database {
        Database(
            self.0
                .upgrade()
                .expect("Database was dropped while Region still exists"),
        )
    }
}
