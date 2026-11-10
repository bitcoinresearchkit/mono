use std::collections::BTreeSet;

use super::{MutableRawVec, MutableVec};
use crate::{Bytes, Error, ImportOptions, ImportableVec, Result};

impl<V> MutableVec<V>
where
    V: MutableRawVec,
{
    pub(crate) fn import_strict_with(options: ImportOptions) -> Result<Self> {
        let holes = options
            .db
            .region(&Self::holes_region_name_with(options.name))
            .map(|region| {
                let reader = region.create_reader();
                let bytes = reader.read_all();
                if !bytes.len().is_multiple_of(size_of::<usize>()) {
                    return Err(Error::CorruptedRegion {
                        name: Self::holes_region_name_with(options.name),
                        region_len: bytes.len(),
                    });
                }
                bytes
                    .chunks(size_of::<usize>())
                    .map(usize::from_bytes)
                    .collect::<Result<BTreeSet<usize>>>()
            })
            .transpose()?;
        let has_stored_holes = holes.is_some();
        Ok(Self::from_parts(
            V::import_strict_with(options)?,
            holes.unwrap_or_default(),
            has_stored_holes,
        ))
    }
}

impl<V> ImportableVec for MutableVec<V>
where
    V: MutableRawVec,
{
    fn import_with(options: ImportOptions) -> Result<Self> {
        match Self::import_strict_with(options) {
            Err(error) if error.is_schema_mismatch() => {
                options
                    .db
                    .remove_region_if_exists(&Self::holes_region_name_with(options.name))?;
                Ok(Self::from_parts(
                    V::import_with(options)?,
                    BTreeSet::new(),
                    false,
                ))
            }
            result => result,
        }
    }
}
