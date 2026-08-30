use crate::{
    file::{CURRENT_MAGIC, CURRENT_VERSION_FILE, rewrite_atomic},
    version::Version,
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::path::Path;

impl Version {
    pub fn persist(&self, folder: &Path) -> crate::Result<()> {
        log::trace!("Persisting version {} in {}", self.id(), folder.display());

        let mut current = CURRENT_MAGIC.to_vec();
        current.write_u64::<LittleEndian>(self.id())?;
        self.encode_into(&mut current)?;
        rewrite_atomic(&folder.join(CURRENT_VERSION_FILE), &current)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn version_persist_replaces_partial_current() -> crate::Result<()> {
        let directory = tempfile::tempdir()?;
        let version = Version::new(0);
        std::fs::write(directory.path().join(CURRENT_VERSION_FILE), b"partial")?;

        version.persist(directory.path())?;

        let current = std::fs::read(directory.path().join(CURRENT_VERSION_FILE))?;
        assert_ne!(b"partial".as_slice(), current.as_slice());
        Ok(())
    }
}
