use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};
use log::trace;
use xxhash_rust::xxh3;

use crate::{
    Result,
    file::{CURRENT_VERSION_FILE, MAGIC_BYTES, rewrite_atomic},
    version::Version,
};

impl Version {
    pub fn persist(&self, folder: &Path) -> Result<()> {
        trace!("Persisting version {} in {}", self.id(), folder.display());

        let mut current = MAGIC_BYTES.to_vec();
        current.write_u64::<LittleEndian>(self.id())?;
        self.encode_into(&mut current)?;
        let checksum = xxh3::xxh3_128(&current);
        current.write_u128::<LittleEndian>(checksum)?;
        rewrite_atomic(&folder.join(CURRENT_VERSION_FILE), &current)?;

        Ok(())
    }
}
