use std::{
    cmp::Reverse,
    collections::{BinaryHeap, hash_map::Entry},
    path::Path,
    sync::{Arc, atomic::Ordering},
};

use rustc_hash::FxHashMap;

use crate::{Database, Error, Region, RegionMetadata, Result};

use super::metadata_file::MetadataFile;

/// Registry of live regions by ID and vacant metadata slots available for reuse.
pub(crate) struct Regions {
    by_id: FxHashMap<Arc<str>, Region>,
    /// Lowest vacant metadata slot first; avoids rescanning the registry on reuse.
    free_indexes: BinaryHeap<Reverse<usize>>,
    metadata: MetadataFile,
}

impl Regions {
    pub(crate) fn open(parent: &Path) -> Result<Self> {
        Ok(Self {
            by_id: FxHashMap::default(),
            free_indexes: BinaryHeap::new(),
            metadata: MetadataFile::open(&parent.join("regions"))?,
        })
    }

    #[inline]
    pub(crate) fn get(&self, id: &str) -> Option<&Region> {
        self.by_id.get(id)
    }

    fn iter(&self) -> impl Iterator<Item = &Region> {
        self.by_id.values()
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = &str> {
        self.by_id.keys().map(|id| id.as_ref())
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.by_id.len()
    }

    pub(crate) fn fill(&mut self, db: &Database) -> Result<Vec<(usize, Region)>> {
        let data_len = db.file_len();
        let slots = self.metadata.slots();
        self.by_id.reserve(slots.len());
        let mut imported = Vec::with_capacity(slots.len());
        for (index, meta) in slots.enumerate() {
            let meta = meta.map_err(|error| {
                Error::CorruptedMetadata(format!("region slot {index}: {error}"))
            })?;
            let Some(meta) = meta else {
                self.free_indexes.push(Reverse(index));
                continue;
            };
            let end = meta
                .start()
                .checked_add(meta.reserved())
                .ok_or_else(|| Error::CorruptedMetadata("region end overflows".to_string()))?;
            if end > data_len {
                return Err(Error::CorruptedMetadata(format!(
                    "region slot {index} ends at {end}, beyond data file length {data_len}"
                )));
            }
            let Entry::Vacant(entry) = self.by_id.entry(meta.shared_id()) else {
                return Err(Error::CorruptedMetadata(format!(
                    "duplicate region id '{}'",
                    meta.id()
                )));
            };
            let start = meta.start();
            let region = Region::new(db, index, meta);
            region.0.tail_needs_punch.store(true, Ordering::Relaxed);
            entry.insert(region.clone());
            imported.push((start, region));
        }

        Ok(imported)
    }

    pub(crate) fn shrink_to_fit(&mut self) -> Result<()> {
        let len = self
            .iter()
            .map(|region| region.index() + 1)
            .max()
            .unwrap_or(0);
        self.free_indexes.retain(|&Reverse(index)| index < len);
        self.metadata.truncate(len)
    }

    pub(crate) fn create(&mut self, db: &Database, id: Arc<str>, start: usize) -> Result<Region> {
        assert!(
            !self.by_id.contains_key(id.as_ref()),
            "region ID is already registered"
        );
        // With no vacancies, the live slots are contiguous from zero.
        let index = self
            .free_indexes
            .peek()
            .map_or(self.len(), |&Reverse(index)| index);

        let meta = RegionMetadata::new(id.clone(), start);
        let region = Region::new(db, index, meta);

        self.metadata.reserve(index + 1)?;

        // Reserve succeeded before committing any change to the vacancy index.
        self.free_indexes.pop();
        self.by_id.insert(id, region.clone());

        self.metadata.write(index, &region.meta());

        Ok(region)
    }

    pub(crate) fn check_removable(&self, region: &Region) -> Result<()> {
        if !self
            .get(region.meta().id())
            .is_some_and(|stored| stored.ptr_eq(region))
        {
            return Err(Error::RegionNotFound);
        }
        // One handle each in the caller, Regions, and Layout. Their write locks
        // prevent new handles from being obtained during this check/removal.
        let ref_count = Arc::strong_count(&region.0);
        if ref_count > 3 {
            return Err(Error::RegionStillReferenced {
                id: region.meta().id().to_string(),
                ref_count,
            });
        }
        Ok(())
    }

    pub(crate) fn remove(&mut self, region: &Region) {
        self.free_indexes.push(Reverse(region.index()));
        self.by_id.remove(region.meta().id());
        self.metadata.clear(region.index());
    }

    pub(crate) fn flush(&self) -> Result<bool> {
        self.metadata.flush()
    }

    pub(crate) fn update_bounds(&self, index: usize, meta: &RegionMetadata) {
        self.metadata.update_bounds(index, meta);
    }
}
