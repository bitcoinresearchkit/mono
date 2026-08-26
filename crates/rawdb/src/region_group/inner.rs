use parking_lot::Mutex;

use crate::region_metadata::MAX_RESERVED_SIZE;
use crate::{Error, PAGE_SIZE, Region, Result};

#[derive(Debug)]
pub(crate) struct RegionGroupInner {
    regions: Box<[Region]>,
    relocation_lock: Mutex<()>,
}

impl RegionGroupInner {
    pub(crate) fn new(regions: &[Region]) -> Result<Self> {
        let group = Self {
            regions: regions.into(),
            relocation_lock: Mutex::new(()),
        };
        group.consolidate()?;
        Ok(group)
    }

    pub(crate) fn matches(&self, regions: &[Region]) -> bool {
        self.regions
            .iter()
            .map(Region::index)
            .eq(regions.iter().map(Region::index))
    }

    fn consolidate(&self) -> Result<()> {
        let reservations = self
            .regions
            .iter()
            .map(|region| region.meta().reserved())
            .collect::<Vec<_>>();
        if self.is_contiguous(&reservations) {
            return Ok(());
        }
        self.relocate(&reservations)
    }

    pub(crate) fn reserve(&self, region: &Region, capacity: usize) -> Result<()> {
        let _guard = self.relocation_lock.lock();
        let current = region.meta().reserved();
        if capacity <= current {
            return Ok(());
        }

        let mut multiplier = 2usize;
        while current
            .checked_mul(multiplier)
            .is_some_and(|reserved| reserved < capacity)
        {
            multiplier = multiplier.checked_mul(2).ok_or(Error::RegionSizeOverflow {
                current,
                requested: capacity,
            })?;
        }

        let mut found = false;
        let reservations = self
            .regions
            .iter()
            .map(|member| {
                found |= member.ptr_eq(region);
                let reserved = member.meta().reserved();
                reserved
                    .checked_mul(multiplier)
                    .filter(|reserved| *reserved <= MAX_RESERVED_SIZE)
                    .ok_or(Error::RegionSizeOverflow {
                        current: reserved,
                        requested: capacity,
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        if !found {
            return Err(Error::RegionGroupMemberNotFound);
        }
        self.relocate(&reservations)
    }

    fn is_contiguous(&self, reservations: &[usize]) -> bool {
        let mut expected = self.regions[0].meta().start();
        self.regions
            .iter()
            .zip(reservations)
            .all(|(region, &reserved)| {
                let meta = region.meta();
                let matches = meta.start() == expected && meta.reserved() == reserved;
                expected = expected.saturating_add(reserved);
                matches
            })
    }

    fn relocate(&self, reservations: &[usize]) -> Result<()> {
        debug_assert_eq!(self.regions.len(), reservations.len());
        let db = self.regions[0].db();
        let total = reservations.iter().try_fold(0usize, |total, &reserved| {
            total
                .checked_add(reserved)
                .ok_or(Error::RegionSizeOverflow {
                    current: total,
                    requested: reserved,
                })
        })?;
        debug_assert!(total >= PAGE_SIZE);

        let snapshots = self
            .regions
            .iter()
            .map(|region| {
                let meta = region.meta();
                (meta.start(), meta.len())
            })
            .collect::<Vec<_>>();

        let mut layout = db.layout_mut();
        let target = if let Some(start) = layout.find_smallest_adequate_hole(total) {
            layout.remove_or_compress_hole(start, total)?;
            start
        } else {
            layout.len()
        };
        layout.reserve(target, total);
        drop(layout);

        if let Err(error) = db.set_min_len(target + total) {
            db.layout_mut().take_reserved(target);
            return Err(error);
        }

        let mut offset = 0usize;
        for (&(start, len), &reserved) in snapshots.iter().zip(reservations) {
            db.copy(start, target + offset, len)?;
            offset += reserved;
        }

        let mut layout = db.layout_mut();
        for region in &self.regions {
            layout.remove_region(region)?;
        }
        assert_eq!(layout.take_reserved(target), Some(total));

        let mut offset = 0usize;
        for (region, &reserved) in self.regions.iter().zip(reservations) {
            let mut meta = region.meta_mut();
            meta.set_start(target + offset);
            meta.set_reserved(reserved);
            drop(meta);
            layout.insert_region(target + offset, region);
            offset += reserved;
        }
        drop(layout);

        let regions = db.regions();
        for (region, &(_, len)) in self.regions.iter().zip(&snapshots) {
            if len > 0 {
                region.mark_dirty(0, len);
            }
            region.meta_mut().write_if_dirty(region.index(), &regions);
        }
        Ok(())
    }
}
