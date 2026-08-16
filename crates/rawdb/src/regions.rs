use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::Path,
    sync::Arc,
};

use log::debug;
use memmap2::MmapMut;
use parking_lot::Mutex;

use crate::{
    Database, Error, PAGE_SIZE, RegionMetadata, Result, SIZE_OF_REGION_METADATA, create_mmap,
    region::Region, write_to_mmap,
};

#[derive(Debug)]
pub struct Regions {
    id_to_index: HashMap<String, usize>,
    index_to_region: Vec<Option<Region>>,
    file: File,
    mmap: MmapMut,
    /// Whether the metadata mmap differs from its durable file.
    dirty: Mutex<bool>,
}

impl Regions {
    pub fn open(parent: &Path) -> Result<Self> {
        fs::create_dir_all(parent)?;

        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .truncate(false)
            .open(parent.join("regions"))?;
        file.try_lock()?;

        let mmap = create_mmap(&file)?;

        Ok(Self {
            id_to_index: HashMap::new(),
            index_to_region: vec![],
            file,
            mmap,
            dirty: Mutex::new(false),
        })
    }

    fn file_len(&self) -> Result<usize> {
        Ok(self.file.metadata()?.len() as usize)
    }

    pub(crate) fn fill(&mut self, db: &Database) -> Result<()> {
        let file_len = self.file_len()?;

        if file_len % SIZE_OF_REGION_METADATA != 0 {
            return Err(Error::CorruptedMetadata(format!(
                "regions file size {} is not a multiple of {}",
                file_len, SIZE_OF_REGION_METADATA
            )));
        }

        let num_slots = file_len / SIZE_OF_REGION_METADATA;

        self.index_to_region
            .resize_with(num_slots, Default::default);

        for index in 0..num_slots {
            let start = index * SIZE_OF_REGION_METADATA;
            let bytes = &self.mmap[start..start + SIZE_OF_REGION_METADATA];

            let Ok(meta) = RegionMetadata::from_bytes(bytes) else {
                continue;
            };

            self.id_to_index.insert(meta.id().to_string(), index);
            self.index_to_region[index] = Some(Region::from(db, index, meta));
        }

        Ok(())
    }

    pub(crate) fn set_min_len(&mut self, len: usize) -> Result<()> {
        let file_len = self.file_len()?;
        if file_len < len {
            self.file.set_len(len as u64)?;
            self.mmap = create_mmap(&self.file)?;
        }
        Ok(())
    }

    pub(crate) fn create(&mut self, db: &Database, id: String, start: usize) -> Result<Region> {
        let index = self
            .index_to_region
            .iter()
            .enumerate()
            .find(|(_, opt)| opt.is_none())
            .map(|(index, _)| index)
            .unwrap_or_else(|| self.index_to_region.len());

        let region = Region::new(db, id.clone(), index, start, 0, PAGE_SIZE);

        self.set_min_len((index + 1) * SIZE_OF_REGION_METADATA)?;

        let region_opt = Some(region.clone());
        if index < self.index_to_region.len() {
            self.index_to_region[index] = region_opt
        } else {
            self.index_to_region.push(region_opt);
        }

        if self.id_to_index.insert(id, index).is_some() {
            return Err(Error::RegionAlreadyExists);
        }

        region.meta_mut().write_if_dirty(index, self);

        Ok(region)
    }

    #[inline]
    pub fn get_from_index(&self, index: usize) -> Option<&Region> {
        self.index_to_region.get(index).and_then(Option::as_ref)
    }

    #[inline]
    pub fn get_from_id(&self, id: &str) -> Option<&Region> {
        self.id_to_index
            .get(id)
            .and_then(|&index| self.get_from_index(index))
    }

    pub(crate) fn rename(&mut self, old_id: &str, new_id: &str) -> Result<()> {
        let index = self
            .id_to_index
            .get(old_id)
            .copied()
            .ok_or(Error::RegionNotFound)?;

        if self.id_to_index.contains_key(new_id) {
            return Err(Error::RegionAlreadyExists);
        }

        self.id_to_index.remove(old_id);
        self.id_to_index.insert(new_id.to_string(), index);

        Ok(())
    }

    pub(crate) fn remove(&mut self, region: &Region) -> Result<()> {
        // Expected 2: one from caller, one from self.index_to_region.
        let ref_count = Arc::strong_count(region.arc());
        debug!(
            "regions.remove '{}': arc count = {} (expected <= 2)",
            region.meta().id(),
            ref_count
        );
        if ref_count > 2 {
            return Err(Error::RegionStillReferenced {
                id: region.meta().id().to_string(),
                ref_count,
            });
        }

        if self
            .index_to_region
            .get_mut(region.index())
            .and_then(Option::take)
            .is_none()
        {
            return Err(Error::RegionNotFound);
        }

        self.id_to_index.remove(region.meta().id());

        self.write_at(region.index(), &[0u8; SIZE_OF_REGION_METADATA]);

        Ok(())
    }

    /// Makes every completed metadata write preceding this call durable.
    pub(crate) fn flush(&self) -> Result<bool> {
        let mut dirty = self.dirty.lock();
        if !*dirty {
            return Ok(false);
        }

        self.mmap.flush_async()?;
        self.file.sync_data()?;
        *dirty = false;

        Ok(true)
    }

    pub(crate) fn write_at(&self, index: usize, data: &[u8]) {
        debug_assert_eq!(data.len(), SIZE_OF_REGION_METADATA);
        let offset = index * SIZE_OF_REGION_METADATA;
        let mut dirty = self.dirty.lock();
        write_to_mmap(&self.mmap, offset, data);
        *dirty = true;
    }

    #[inline]
    pub fn index_to_region(&self) -> &[Option<Region>] {
        &self.index_to_region
    }

    #[inline]
    pub fn id_to_index(&self) -> &HashMap<String, usize> {
        &self.id_to_index
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.id_to_index.len()
    }
}
