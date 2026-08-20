use std::collections::BTreeSet;

use rawdb::Database;

use crate::{Bytes, Error, ImportOptions, ImportableVec, Version};

use super::{MutableRawVec, MutableVec};

impl<V> MutableVec<V>
where
    V: MutableRawVec,
{
    fn import_inner(options: ImportOptions) -> crate::Result<Self> {
        let holes = options
            .db
            .get_region(&Self::holes_region_name_with(options.name))
            .map(|region| {
                region
                    .create_reader()
                    .read_all()
                    .chunks(size_of::<usize>())
                    .map(usize::from_bytes)
                    .collect::<crate::Result<BTreeSet<usize>>>()
            })
            .transpose()?;
        let has_stored_holes = holes.is_some();
        Ok(Self::from_parts(
            V::import_with(options)?,
            holes.unwrap_or_default(),
            has_stored_holes,
        ))
    }
}

impl<V> ImportableVec for MutableVec<V>
where
    V: MutableRawVec,
{
    fn import(db: &Database, name: &str, version: Version) -> crate::Result<Self> {
        Self::import_with((db, name, version).into())
    }

    fn import_with(options: ImportOptions) -> crate::Result<Self> {
        Self::import_inner(options)
    }

    fn forced_import(db: &Database, name: &str, version: Version) -> crate::Result<Self> {
        Self::forced_import_with((db, name, version).into())
    }

    fn forced_import_with(options: ImportOptions) -> crate::Result<Self> {
        match Self::import_inner(options) {
            Err(Error::WrongEndian)
            | Err(Error::WrongLength { .. })
            | Err(Error::DifferentFormat { .. })
            | Err(Error::DifferentVersion { .. }) => {
                options
                    .db
                    .remove_region_if_exists(&Self::holes_region_name_with(options.name))?;
                Ok(Self::new(V::forced_import_with(options)?))
            }
            result => result,
        }
    }
}
