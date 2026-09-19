use std::{
    collections::{BTreeMap, BTreeSet},
    mem,
};

use log::debug;

use crate::{Error, Region, Result};

use super::regions::Regions;

/// Tracks live allocations and reusable or pending holes in the database file.
#[derive(Default)]
pub(crate) struct Layout {
    start_to_region: BTreeMap<usize, Region>,
    start_to_hole: BTreeMap<usize, usize>,
    /// (size, start) order gives best fit, breaking ties by lowest offset.
    holes_by_size: BTreeSet<(usize, usize)>,
    /// Holes from region moves, reusable after flush.
    pending_holes: BTreeMap<usize, usize>,
    /// Runtime-only reclamation state for reusable holes.
    holes_need_punch: bool,
}

impl TryFrom<&Regions> for Layout {
    type Error = Error;

    fn try_from(regions: &Regions) -> Result<Self> {
        let start_to_region: BTreeMap<usize, Region> = regions
            .iter()
            .map(|region| (region.meta().start(), region.clone()))
            .collect();
        if start_to_region.len() != regions.len() {
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
                let size = start - prev_end;
                layout.insert_hole(prev_end, size);
            }
            prev_end = start + region.meta().reserved();
        }

        layout.start_to_region = start_to_region;
        // The unused file tail can contain data from regions removed before reopen.
        layout.holes_need_punch = true;
        Ok(layout)
    }
}

impl Layout {
    /// Live regions in allocation order, independently of metadata slot reuse.
    pub(crate) fn regions(&self) -> impl Iterator<Item = &Region> {
        self.start_to_region.values()
    }

    fn insert_hole(&mut self, start: usize, size: usize) {
        self.start_to_hole.insert(start, size);
        self.holes_by_size.insert((size, start));
    }

    fn remove_hole(&mut self, start: usize) -> Option<usize> {
        let size = self.start_to_hole.remove(&start)?;

        self.holes_by_size.remove(&(size, start));

        Some(size)
    }

    #[cfg(test)]
    pub(crate) fn start_to_region(&self) -> &BTreeMap<usize, Region> {
        &self.start_to_region
    }

    pub(crate) fn start_to_hole(&self) -> &BTreeMap<usize, usize> {
        &self.start_to_hole
    }

    #[inline]
    pub(crate) fn holes_need_punch(&self) -> bool {
        self.holes_need_punch
    }

    #[inline]
    pub(crate) fn mark_holes_punched(&mut self) {
        self.holes_need_punch = false;
    }

    pub(crate) fn end(&self) -> usize {
        let mut len = 0;
        if let Some((&start, &size)) = self.start_to_hole.last_key_value() {
            len = len.max(start + size);
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
        assert!(self.start_to_region.insert(start, region.clone()).is_none())
    }

    pub(crate) fn move_region(&mut self, new_start: usize, region: &Region) {
        self.remove_region(region);
        self.insert_region(new_start, region);
    }

    pub(crate) fn remove_region(&mut self, region: &Region) {
        let region_meta = region.meta();
        let start = region_meta.start();
        let reserved = region_meta.reserved();

        assert!(
            self.start_to_region
                .get(&start)
                .is_some_and(|stored| stored.ptr_eq(region)),
            "region allocation must match its registered handle"
        );

        self.start_to_region.remove(&start);
        self.pending_holes.insert(start, reserved);
    }

    pub(crate) fn get_hole(&self, start: usize) -> Option<usize> {
        self.start_to_hole.get(&start).copied()
    }

    pub(crate) fn find_smallest_adequate_hole(&self, min_size: usize) -> Option<usize> {
        self.holes_by_size
            .range((min_size, 0)..)
            .next()
            .map(|&(_, start)| start)
    }

    pub(crate) fn consume_hole(&mut self, start: usize, len: usize) {
        let size = self
            .remove_hole(start)
            .expect("allocation must refer to a reusable hole");
        assert!(len <= size, "allocation must fit its hole");
        if len < size {
            self.insert_hole(start + len, size - len);
        }
    }

    pub(crate) fn promote_pending_holes(&mut self, name: &str) {
        let count = self.pending_holes.len();
        if count > 0 {
            debug!("{}: promoted {} pending holes", name, count);
        }
        for (start, mut size) in mem::take(&mut self.pending_holes) {
            let mut final_start = start;

            // Coalesce with adjacent real hole BEFORE
            if let Some((&hole_start, &hole_size)) = self.start_to_hole.range(..start).next_back()
                && hole_start + hole_size == start
            {
                self.remove_hole(hole_start);
                final_start = hole_start;
                size += hole_size;
            }

            // Coalesce with adjacent real hole AFTER
            if let Some(hole_after_size) = self.remove_hole(final_start + size) {
                size += hole_after_size;
            }

            self.insert_hole(final_start, size);
        }
        self.holes_need_punch |= count > 0;
    }
}
