use std::collections::BTreeSet;

use super::{MutableRawVec, MutableVec};
use crate::{Bytes, Error, ImportOptions, ImportableVec, Result};

impl<V> MutableVec<V>
where
    V: MutableRawVec,
{
    fn import_inner(options: ImportOptions) -> Result<Self> {
        let holes = options
            .db
            .get_region(&Self::holes_region_name_with(options.name))
            .map(|region| {
                region
                    .create_reader()
                    .read_all()
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
        match Self::import_inner(options) {
            Err(Error::WrongLength { .. })
            | Err(Error::DifferentFormat { .. })
            | Err(Error::DifferentValueSize { .. })
            | Err(Error::DifferentVersion { .. }) => {
                options
                    .db
                    .remove_region_if_exists(&Self::holes_region_name_with(options.name))?;
                Ok(Self::new(V::import_with(options)?))
            }
            result => result,
        }
    }
}
