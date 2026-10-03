use super::EagerVec;
use crate::{ImportOptions, ImportableVec, Result};

impl<V: ImportableVec> ImportableVec for EagerVec<V> {
    fn import_with(options: ImportOptions) -> Result<Self> {
        Ok(Self(V::import_with(options)?))
    }
}
