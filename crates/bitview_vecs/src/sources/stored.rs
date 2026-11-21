use std::{collections::BTreeMap, path::PathBuf};

use bitview_primitives::Day1;
use brk_error::Result;
use brk_types::{Height, Version};
use rangeindex::RangeMap;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyStoredVec, AnyVec, Budgeted, Database, Formattable, Header, ImportableVec, PcoVec,
    PcoVecValue, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Region, Rw, Stamp,
    StorageMode, StoredVec, TypedVec, WritableVec,
};

use crate::{IndexSources, RangeMapVec, read_sorted_optional};

type HeightVec<T> = PcoVec<Height, T, Budgeted>;
type DayCloses<T> = PcoVec<Day1, T>;

/// One stored per-block series and its shared, budgeted cache.
pub type CachedSeries<T, M = Rw> = <M as StorageMode>::Stored<StoredSeries<T>>;

pub fn import_cached<T: PcoVecValue>(
    db: &Database,
    name: &str,
    version: Version,
    indexes: &IndexSources,
) -> Result<CachedSeries<T>> {
    Ok(StoredSeries {
        height: PcoVec::import(db, name, version)?,
        day_closes: PcoVec::import(db, name, version)?,
        day_starts: indexes.first_height.day1.clone(),
    })
}

/// A per-block series stored with its value at the last block of every closed day.
///
/// A day, week or month view asks for one block per period, and a compressed page holds about a
/// week of blocks, so reading those from the per-block vector decodes nearly all of it. The day
/// closes are the same values in a few pages. Readers take a closed day's last block from them
/// and every other block from the per-block vector ([`SeriesReader`]).
pub struct StoredSeries<T: PcoVecValue> {
    height: HeightVec<T>,
    /// Row `day`: `height` at the last block of `day`, for the days the stored blocks close.
    day_closes: DayCloses<T>,
    day_starts: RangeMapVec<Day1, Height>,
}

/// The most day rows a series holds for after the publication: an update at the tip closes one
/// day. Anything more (a sync, a catch-up after an outage, the first write over data from before
/// the day rows) writes its rows with the blocks.
const HELD_DAYS: usize = 1;

/// The days `len` blocks close. A day's last block is known once the next day's first block
/// exists, so that is every day `len` reaches but the last.
fn closed_days(day_starts: &RangeMap<Height, Day1>, len: usize) -> usize {
    day_starts.reached(len).saturating_sub(1)
}

impl<T: PcoVecValue> StoredSeries<T> {
    /// Runs one `write` of the per-block vector and keeps the day closes in step with it. Closes
    /// the blocks on disk no longer back (after a truncation, reset or rollback) go first, so a
    /// stop between the writes leaves the day closes short, which readers fall back from, never
    /// stale. The days the write closes are held for [`AnyStoredVec::write_held`], which runs
    /// once the update is published: their rows land in pages last written a day ago, and reading
    /// those back from disk must not delay the block. Readers take those blocks from the per-block
    /// vector until then.
    fn write_with<R>(
        &mut self,
        write: impl FnOnce(&mut HeightVec<T>) -> vecdb::Result<R>,
    ) -> vecdb::Result<R> {
        let day_starts = self.day_starts.load();
        let stored = self.height.stored_len();
        let kept = closed_days(&day_starts, stored);
        if kept < self.day_closes.len() {
            self.day_closes.truncate_if_needed_at(kept)?;
            self.day_closes.write()?;
        }

        let pushed = self.height.pushed();
        let closing = self.day_closes.len()..closed_days(&day_starts, stored + pushed.len());
        if closing.is_empty() {
            return write(&mut self.height);
        }
        // A day before the first block has no last block: its row is never read and holds the
        // first block's value.
        let last_blocks = closing
            .map(|day| usize::from(*day_starts.starts().get(day + 1).unwrap()).max(1) - 1)
            .collect::<Vec<_>>();
        let on_disk = last_blocks.partition_point(|&block| block < stored);
        let (stored_blocks, pushed_blocks) = last_blocks.split_at(on_disk);
        // One sorted read, whatever the number of days: the first write over existing data
        // closes every day of its history.
        for value in read_sorted_optional(
            &self.height,
            stored_blocks.iter().map(|&block| Some(block)),
            true,
        ) {
            self.day_closes.push(value.unwrap());
        }
        for &block in pushed_blocks {
            self.day_closes.push(pushed[block - stored]);
        }
        debug_assert_eq!(
            self.day_closes.len(),
            closed_days(&day_starts, stored + pushed.len()),
            "{}: the day rows pushed do not end at the last day the blocks close",
            self.height.name()
        );

        let result = write(&mut self.height)?;
        if self.day_closes.pushed().len() > HELD_DAYS {
            self.day_closes.write()?;
        }
        Ok(result)
    }
}

impl<T: PcoVecValue> AnyVec for StoredSeries<T> {
    fn version(&self) -> Version {
        self.height.version()
    }

    fn name(&self) -> &str {
        self.height.name()
    }

    fn len(&self) -> usize {
        self.height.len()
    }

    fn is_mutable(&self) -> bool {
        self.height.is_mutable()
    }

    fn region_names(&self) -> Vec<String> {
        [self.height.region_names(), self.day_closes.region_names()].concat()
    }

    vecdb::any_vec_type_methods!(Height, T);
}

impl<T: PcoVecValue> TypedVec for StoredSeries<T> {
    type I = Height;
    type T = T;
}

impl<T: PcoVecValue> AnyStoredVec for StoredSeries<T> {
    fn db_path(&self) -> PathBuf {
        self.height.db_path()
    }

    fn region(&self) -> &Region {
        self.height.region()
    }

    fn header(&self) -> &Header {
        self.height.header()
    }

    fn mut_header(&mut self) -> &mut Header {
        self.height.mut_header()
    }

    fn saved_stamped_changes(&self) -> u16 {
        self.height.saved_stamped_changes()
    }

    fn write(&mut self) -> vecdb::Result<bool> {
        self.write_with(AnyStoredVec::write)
    }

    fn write_held(&mut self) -> vecdb::Result<bool> {
        if self.day_closes.pushed().is_empty() {
            return Ok(false);
        }
        self.day_closes.write()?;
        Ok(true)
    }

    fn db(&self) -> Database {
        self.height.db()
    }

    fn real_stored_len(&self) -> usize {
        self.height.real_stored_len()
    }

    fn stored_len(&self) -> usize {
        self.height.stored_len()
    }

    fn any_save_changes(&self, stamp: Stamp) -> vecdb::Result<()> {
        self.height.any_save_changes(stamp)
    }

    fn any_write_checkpoint(&mut self, stamp: Stamp) -> vecdb::Result<()> {
        self.write_with(|height| height.any_write_checkpoint(stamp))
    }

    fn any_remove_changes_above(&self, stamp: Stamp) -> vecdb::Result<()> {
        self.height.any_remove_changes_above(stamp)
    }

    fn serialize_changes(&self) -> vecdb::Result<Vec<u8>> {
        self.height.serialize_changes()
    }

    fn remove(self) -> vecdb::Result<()> {
        self.day_closes.remove()?;
        self.height.remove()
    }

    fn any_truncate_if_needed_at(&mut self, index: usize) -> vecdb::Result<()> {
        self.height.any_truncate_if_needed_at(index)
    }

    fn any_reset(&mut self) -> vecdb::Result<()> {
        self.height.any_reset()
    }
}

impl<T: PcoVecValue> WritableVec<Height, T> for StoredSeries<T> {
    #[inline]
    fn push(&mut self, value: T) {
        self.height.push(value);
    }

    #[inline]
    fn pushed(&self) -> &[T] {
        self.height.pushed()
    }

    fn truncate_if_needed_at(&mut self, index: usize) -> vecdb::Result<()> {
        self.height.truncate_if_needed_at(index)
    }

    fn reset(&mut self) -> vecdb::Result<()> {
        self.height.reset()
    }

    fn reset_unsaved(&mut self) {
        self.height.reset_unsaved();
    }

    fn is_dirty(&self) -> bool {
        self.height.is_dirty()
    }

    fn rollback(&mut self) -> vecdb::Result<()> {
        self.height.rollback()
    }

    fn find_rollback_files(&self) -> vecdb::Result<BTreeMap<Stamp, PathBuf>> {
        self.height.find_rollback_files()
    }

    fn save_rollback_state(&mut self) {
        self.height.save_rollback_state();
    }
}

/// The [`ReadableVec`] methods a series passes straight to its per-block vector.
macro_rules! per_block_reads {
    () => {
        #[inline]
        fn read_cached_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
            self.height.read_cached_into_at(from, to, out)
        }

        #[inline]
        fn cursor_chunk_size(&self) -> usize {
            self.height.cursor_chunk_size()
        }

        #[inline]
        fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
            self.height.read_into_at(from, to, buf);
        }

        fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
            self.height.for_each_chunk_at(from, to, f);
        }

        fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
            self.height.for_each_range_dyn_at(from, to, f);
        }

        #[inline]
        fn try_fold_range_at<B, E, F: FnMut(B, T) -> std::result::Result<B, E>>(
            &self,
            from: usize,
            to: usize,
            init: B,
            f: F,
        ) -> std::result::Result<B, E> {
            self.height.try_fold_range_at(from, to, init, f)
        }

        /// One block is one page either way.
        #[inline]
        fn collect_one_at(&self, index: usize) -> Option<T> {
            self.height.collect_one_at(index)
        }
    };
}

/// The writer reads its own rows from the per-block vector alone.
impl<T: PcoVecValue> ReadableVec<Height, T> for StoredSeries<T> {
    per_block_reads!();

    #[inline]
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        self.height.read_sorted_into_at(indices, out);
    }
}

impl<T: PcoVecValue> ReadableCloneableVec<Height, T> for StoredSeries<T> {
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<Height, T> {
        StoredVec::read_only_clone(self).into_read_only_boxed()
    }
}

impl<T: PcoVecValue> StoredVec for StoredSeries<T> {
    type ReadOnly = SeriesReader<T>;

    fn read_only_clone(&self) -> SeriesReader<T> {
        SeriesReader {
            height: StoredVec::read_only_clone(&self.height),
            day_closes: StoredVec::read_only_clone(&self.day_closes),
            day_starts: self.day_starts.clone(),
        }
    }
}

/// Reads a [`StoredSeries`]: every read returns the per-block vector's values, and a sorted read
/// takes the blocks that end a closed day from the day closes.
#[derive(Clone)]
pub struct SeriesReader<T: PcoVecValue> {
    height: <HeightVec<T> as StoredVec>::ReadOnly,
    day_closes: <DayCloses<T> as StoredVec>::ReadOnly,
    day_starts: RangeMapVec<Day1, Height>,
}

impl<T: PcoVecValue> AnyVec for SeriesReader<T> {
    fn version(&self) -> Version {
        self.height.version()
    }

    fn name(&self) -> &str {
        self.height.name()
    }

    fn len(&self) -> usize {
        self.height.len()
    }

    fn is_mutable(&self) -> bool {
        self.height.is_mutable()
    }

    fn region_names(&self) -> Vec<String> {
        [self.height.region_names(), self.day_closes.region_names()].concat()
    }

    vecdb::any_vec_type_methods!(Height, T);
}

impl<T: PcoVecValue> TypedVec for SeriesReader<T> {
    type I = Height;
    type T = T;
}

impl<T: PcoVecValue> ReadableVec<Height, T> for SeriesReader<T> {
    per_block_reads!();

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let day_starts = self.day_starts.load();
        let starts = day_starts.starts();
        // A close is read only for a day both vectors show closed: the open day, a day whose
        // close is not written yet and a reader bounded below the writer take the block itself.
        let closed =
            closed_days(&day_starts, self.height.visible_len()).min(self.day_closes.visible_len());
        let (Some(&first), true) = (indices.first(), closed > 0) else {
            return self.height.read_sorted_into_at(indices, out);
        };
        // Day `next - 1` ends at `block` when day `next` starts right after it.
        let mut next =
            starts.partition_point_in(0..closed + 1, |&start| usize::from(start) <= first);
        let mut days = Vec::new();
        let mut blocks = Vec::new();
        let from_days = indices
            .iter()
            .map(|&block| {
                while next <= closed && usize::from(*starts.get(next).unwrap()) <= block {
                    next += 1;
                }
                let ends_day = (1..=closed).contains(&next)
                    && usize::from(*starts.get(next).unwrap()) == block + 1;
                if ends_day {
                    days.push(next - 1);
                } else {
                    blocks.push(block);
                }
                ends_day
            })
            .collect::<Vec<_>>();
        if days.is_empty() {
            return self.height.read_sorted_into_at(indices, out);
        }

        let mut days = self.day_closes.read_sorted_at(&days).into_iter();
        let mut blocks = self.height.read_sorted_at(&blocks).into_iter();
        out.reserve(indices.len());
        // The per-block vector stops at its visible length, and so does the output.
        out.extend(from_days.into_iter().map_while(|from_day| {
            if from_day { days.next() } else { blocks.next() }
        }));
    }
}

bitview_traversable::leaf!(stored [T] StoredSeries<T> where
    T: PcoVecValue + Formattable + Serialize + JsonSchema,
);

bitview_traversable::leaf!([T] SeriesReader<T> where
    T: PcoVecValue + Formattable + Serialize + JsonSchema,
);
