use std::{collections::BTreeMap, mem};

use log::debug;
use smallvec::SmallVec;

use crate::{Error, Region, Regions, Result};

/// Tracks regions, holes, and reservations in the database file.
#[derive(Debug, Default)]
pub struct Layout {
    start_to_region: BTreeMap<usize, Region>,
    start_to_hole: BTreeMap<usize, usize>,
    /// hole_size → starts: enables O(log n) best-fit search.
    hole_to_starts: BTreeMap<usize, SmallVec<[usize; 1]>>,
    start_to_reserved: BTreeMap<usize, usize>,
    /// Holes from region moves, reusable after flush.
    pending_holes: BTreeMap<usize, usize>,
    /// Runtime-only reclamation state for reusable holes.
    holes_need_punch: bool,
}

impl From<&Regions> for Layout {
    fn from(regions: &Regions) -> Self {
        let start_to_region: BTreeMap<usize, Region> = regions
            .index_to_region()
            .iter()
            .flatten()
            .map(|region| (region.meta().start(), region.clone()))
            .collect();

        let mut layout = Self {
            start_to_hole: BTreeMap::default(),
            hole_to_starts: BTreeMap::default(),
            start_to_reserved: BTreeMap::default(),
            pending_holes: BTreeMap::default(),
            start_to_region: BTreeMap::default(),
            holes_need_punch: false,
        };

        let mut prev_end = 0;
        for (&start, region) in &start_to_region {
            if prev_end != start {
                let size = start - prev_end;
                layout.insert_hole(prev_end, size);
            }
            prev_end = start + region.meta().reserved();
        }

        layout.start_to_region = start_to_region;
        layout.holes_need_punch = !layout.start_to_hole.is_empty();
        layout
    }
}

impl Layout {
    fn insert_hole(&mut self, start: usize, size: usize) {
        self.start_to_hole.insert(start, size);
        self.hole_to_starts.entry(size).or_default().push(start);
    }

    fn remove_hole(&mut self, start: usize) -> Option<usize> {
        let size = self.start_to_hole.remove(&start)?;

        if let Some(starts) = self.hole_to_starts.get_mut(&size) {
            starts.retain(|s| *s != start);
            if starts.is_empty() {
                self.hole_to_starts.remove(&size);
            }
        }

        Some(size)
    }

    pub fn start_to_region(&self) -> &BTreeMap<usize, Region> {
        &self.start_to_region
    }

    pub fn start_to_hole(&self) -> &BTreeMap<usize, usize> {
        &self.start_to_hole
    }

    #[inline]
    pub fn holes_need_punch(&self) -> bool {
        self.holes_need_punch
    }

    #[inline]
    pub fn mark_holes_punched(&mut self) {
        self.holes_need_punch = false;
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        if let Some((start, reserved)) = self.get_last_reserved() {
            len = len.max(start + reserved);
        }
        if let Some((start, gap)) = self.get_last_hole() {
            len = len.max(start + gap);
        }
        if let Some((&start, &size)) = self.pending_holes.last_key_value() {
            len = len.max(start + size);
        }
        if let Some((start, region)) = self.get_last_region() {
            len = len.max(start + region.meta().reserved());
        }
        len
    }

    pub fn get_last_region(&self) -> Option<(usize, &Region)> {
        self.start_to_region
            .last_key_value()
            .map(|(start, region)| (*start, region))
    }

    fn get_last_hole(&self) -> Option<(usize, usize)> {
        self.start_to_hole
            .last_key_value()
            .map(|(start, gap)| (*start, *gap))
    }

    fn get_last_reserved(&self) -> Option<(usize, usize)> {
        self.start_to_reserved
            .last_key_value()
            .map(|(start, reserved)| (*start, *reserved))
    }

    pub fn is_last_anything(&self, region: &Region) -> bool {
        let Some((last_start, last_region)) = self.get_last_region() else {
            return false;
        };

        last_region.index() == region.index()
            && self
                .get_last_hole()
                .is_none_or(|(hole_start, _)| last_start > hole_start)
            && self
                .get_last_reserved()
                .is_none_or(|(reserved_start, _)| last_start > reserved_start)
            && self
                .pending_holes
                .last_key_value()
                .is_none_or(|(&pending_start, _)| last_start > pending_start)
    }

    pub fn insert_region(&mut self, start: usize, region: &Region) {
        assert!(self.start_to_region.insert(start, region.clone()).is_none())
    }

    pub fn move_region(&mut self, new_start: usize, region: &Region) -> Result<()> {
        self.remove_region(region)?;
        self.insert_region(new_start, region);
        Ok(())
    }

    pub fn remove_region(&mut self, region: &Region) -> Result<()> {
        let region_meta = region.meta();
        let start = region_meta.start();
        let reserved = region_meta.reserved();

        let removed = self.start_to_region.remove(&start);

        if removed
            .as_ref()
            .is_none_or(|region_| region.index() != region_.index())
        {
            return Err(Error::RegionIndexMismatch);
        }

        self.pending_holes.insert(start, reserved);

        Ok(())
    }

    pub fn get_hole(&self, start: usize) -> Option<usize> {
        self.start_to_hole.get(&start).copied()
    }

    pub fn find_smallest_adequate_hole(&self, min_size: usize) -> Option<usize> {
        self.hole_to_starts
            .range(min_size..)
            .next()
            .and_then(|(_, starts)| starts.first().copied())
    }

    pub fn remove_or_compress_hole(&mut self, start: usize, compress_by: usize) -> Result<()> {
        let Some(size) = self.remove_hole(start) else {
            return Ok(());
        };

        if size == compress_by {
            Ok(())
        } else if size > compress_by {
            let new_start = start + compress_by;
            let new_size = size - compress_by;
            self.insert_hole(new_start, new_size);
            Ok(())
        } else {
            Err(Error::HoleTooSmall {
                hole_size: size,
                requested: compress_by,
            })
        }
    }

    pub fn reserve(&mut self, start: usize, reserved: usize) {
        if self.start_to_reserved.insert(start, reserved).is_some() {
            unreachable!();
        }
    }

    pub fn take_reserved(&mut self, start: usize) -> Option<usize> {
        self.start_to_reserved.remove(&start)
    }

    pub fn promote_pending_holes(&mut self, name: &str) {
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
