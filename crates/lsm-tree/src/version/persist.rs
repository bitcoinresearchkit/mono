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

#[cfg(test)]
mod tests {
    use std::fs;

    use byteorder::ByteOrder;
    use tempfile::tempdir;
    use test_log::test;

    use super::*;

    #[test]
    fn version_persist_replaces_partial_current() -> Result<()> {
        let directory = tempdir()?;
        let version = Version::new(0);
        fs::write(directory.path().join(CURRENT_VERSION_FILE), b"partial")?;

        version.persist(directory.path())?;

        let current = fs::read(directory.path().join(CURRENT_VERSION_FILE))?;
        assert_ne!(b"partial".as_slice(), current.as_slice());
        let (payload, checksum) = current.split_at(current.len() - size_of::<u128>());
        assert_eq!(xxh3::xxh3_128(payload), LittleEndian::read_u128(checksum),);
        Ok(())
    }
}
