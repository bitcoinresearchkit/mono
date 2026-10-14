use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
};

use crate::{Error, Region, Result};

/// Tracks live allocations and reusable or pending holes in the database file.
#[derive(Default)]
pub(crate) struct Layout {
    start_to_region: BTreeMap<usize, Region>,
    start_to_hole: BTreeMap<usize, Hole>,
    /// (size, start) order gives best fit, breaking ties by lowest offset.
    holes_by_size: BTreeSet<(usize, usize)>,
    /// Holes from region moves, reusable after flush.
    pending_holes: BTreeMap<usize, usize>,
    /// The file past the layout's end can hold data from regions removed before reopening.
    tail_needs_punch: bool,
}

/// A reusable hole and whether its pages may still hold data (runtime-only reclamation state).
#[derive(Clone, Copy)]
struct Hole {
    size: usize,
    needs_punch: bool,
}

impl TryFrom<Vec<(usize, Region)>> for Layout {
    type Error = Error;

    fn try_from(regions: Vec<(usize, Region)>) -> Result<Self> {
        let count = regions.len();
        let start_to_region: BTreeMap<usize, Region> = regions.into_iter().collect();
        if start_to_region.len() != count {
            return Err(Error::CorruptedMetadata(
                "multiple regions have the same start".to_string(),
            ));
        }

        let mut layout = Self::default();

        let mut prev_end = 0;
        for (&start, region) in &start_to_region {
            if start < prev_end {
                return Err(Error::CorruptedMetadata(format!(
                    "region at {start} overlaps previous end {prev_end}"
                )));
            }
            if prev_end < start {
                layout.insert_hole(prev_end, start - prev_end, true);
            }
            prev_end = start + region.meta().reserved();
        }

        layout.start_to_region = start_to_region;
        layout.tail_needs_punch = true;
        Ok(layout)
    }
}

impl Layout {
    /// Live regions in allocation order, independently of metadata slot reuse.
    pub(crate) fn regions(&self) -> impl Iterator<Item = &Region> {
        self.start_to_region.values()
    }

    fn insert_hole(&mut self, start: usize, size: usize, needs_punch: bool) {
        self.start_to_hole.insert(start, Hole { size, needs_punch });
        self.holes_by_size.insert((size, start));
    }

    fn remove_hole(&mut self, start: usize) -> Option<Hole> {
        let hole = self.start_to_hole.remove(&start)?;
        self.holes_by_size.remove(&(hole.size, start));
        Some(hole)
    }

    /// Punches every hole that may still hold data and, once after opening, the file past the
    /// layout's end (`file_end`). Holes stay marked until their punch succeeds. Returns the
    /// number of punched ranges.
    pub(crate) fn reclaim(
        &mut self,
        file_end: usize,
        mut punch: impl FnMut(usize, usize) -> Result<()>,
    ) -> Result<usize> {
        let mut punched = 0;
        for (&start, hole) in &mut self.start_to_hole {
            if hole.needs_punch {
                punch(start, hole.size)?;
                hole.needs_punch = false;
                punched += 1;
            }
        }
        if self.tail_needs_punch {
            let start = self.end();
            if start < file_end {
                punch(start, file_end - start)?;
                punched += 1;
            }
            self.tail_needs_punch = false;
        }
        Ok(punched)
    }

    pub(crate) fn end(&self) -> usize {
        let mut len = 0;
        if let Some((&start, hole)) = self.start_to_hole.last_key_value() {
            len = len.max(start + hole.size);
        }
        if let Some((&start, &size)) = self.pending_holes.last_key_value() {
            len = len.max(start + size);
        }
        if let Some((&start, region)) = self.start_to_region.last_key_value() {
            len = len.max(start + region.meta().reserved());
        }
        len
    }

    pub(crate) fn insert_region(&mut self, start: usize, region: &Region) {
        let previous = self.start_to_region.insert(start, region.clone());
        debug_assert!(previous.is_none(), "allocation start already registered");
    }

    pub(crate) fn move_region(&mut self, new_start: usize, region: &Region) {
        self.remove_region(region);
        self.insert_region(new_start, region);
    }

    pub(crate) fn remove_region(&mut self, region: &Region) {
        let region_meta = region.meta();
        let start = region_meta.start();
        let reserved = region_meta.reserved();

        debug_assert!(
            self.start_to_region
                .get(&start)
                .is_some_and(|stored| stored.ptr_eq(region)),
            "region allocation must match its registered handle"
        );

        self.start_to_region.remove(&start);
        self.pending_holes.insert(start, reserved);
    }

    pub(crate) fn get_hole(&self, start: usize) -> Option<usize> {
        self.start_to_hole.get(&start).map(|hole| hole.size)
    }

    pub(crate) fn find_smallest_adequate_hole(&self, min_size: usize) -> Option<usize> {
        self.holes_by_size
            .range((min_size, 0)..)
            .next()
            .map(|&(_, start)| start)
    }

    pub(crate) fn consume_hole(&mut self, start: usize, len: usize) {
        let hole = self
            .remove_hole(start)
            .expect("allocation must refer to a reusable hole");
        debug_assert!(len <= hole.size, "allocation must fit its hole");
        if len < hole.size {
            self.insert_hole(start + len, hole.size - len, hole.needs_punch);
        }
    }

    pub(crate) fn promote_pending_holes(&mut self) {
        if self.pending_holes.is_empty() {
            return;
        }
        for (start, mut size) in mem::take(&mut self.pending_holes) {
            let mut final_start = start;

            // Coalesce with adjacent real hole BEFORE
            if let Some((&hole_start, &hole)) = self.start_to_hole.range(..start).next_back()
                && hole_start + hole.size == start
            {
                self.remove_hole(hole_start);
                final_start = hole_start;
                size += hole.size;
            }

            // Coalesce with adjacent real hole AFTER
            if let Some(hole_after) = self.remove_hole(final_start + size) {
                size += hole_after.size;
            }

            // The freed space still holds data: the merged hole is punched whole.
            self.insert_hole(final_start, size, true);
        }
    }
}
