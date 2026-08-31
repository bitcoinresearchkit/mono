mod recovered_table;

pub use recovered_table::RecoveredTable;

use crate::{
    Error, Result,
    file::{CHECKSUMLESS_CURRENT_MAGIC, CURRENT_MAGIC, CURRENT_VERSION_FILE},
    version::DEFAULT_LEVEL_COUNT,
};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::{fs, path::Path};

const TABLE_ENTRY_SIZE: usize = size_of::<u32>() + size_of::<u64>();

pub struct Recovery {
    pub curr_version_id: u64,
    pub table_ids: Vec<Vec<Vec<RecoveredTable>>>,
}

impl Recovery {
    pub fn load(folder: &Path) -> Result<Self> {
        let current_path = folder.join(CURRENT_VERSION_FILE);
        let bytes = fs::read(&current_path)?;
        let magic = bytes
            .get(..CURRENT_MAGIC.len())
            .ok_or(Error::Unrecoverable)?;
        let payload = if magic == CURRENT_MAGIC {
            if bytes.len() < CURRENT_MAGIC.len() + size_of::<u64>() + size_of::<u128>() {
                return Err(Error::Unrecoverable);
            }

            let (payload, checksum) = bytes.split_at(bytes.len() - size_of::<u128>());
            if xxhash_rust::xxh3::xxh3_128(payload) != LittleEndian::read_u128(checksum) {
                log::error!("Current manifest checksum mismatch");
                return Err(Error::Unrecoverable);
            }
            payload
        } else if magic == CHECKSUMLESS_CURRENT_MAGIC {
            if bytes.len() < CHECKSUMLESS_CURRENT_MAGIC.len() + size_of::<u64>() {
                return Err(Error::Unrecoverable);
            }
            bytes.as_slice()
        } else {
            let version = magic.last().copied().ok_or(Error::Unrecoverable)?;
            return Err(Error::InvalidVersion(version));
        };

        let mut reader = payload
            .get(CURRENT_MAGIC.len()..)
            .ok_or(Error::Unrecoverable)?;
        let curr_version_id = reader.read_u64::<LittleEndian>()?;

        log::info!("Recovering current manifest at {}", current_path.display());
        let mut levels = Vec::with_capacity(usize::from(DEFAULT_LEVEL_COUNT));

        for _ in 0..DEFAULT_LEVEL_COUNT {
            let mut level = Vec::new();
            let run_count = reader.read_u8()?;

            for _ in 0..run_count {
                let table_count = reader.read_u32::<LittleEndian>()?;
                let capacity = usize::try_from(table_count).map_err(|_| Error::Unrecoverable)?;
                if capacity == 0 || capacity > reader.len() / TABLE_ENTRY_SIZE {
                    return Err(Error::Unrecoverable);
                }
                let mut run = Vec::with_capacity(capacity);

                for _ in 0..table_count {
                    let id = reader.read_u32::<LittleEndian>()?;
                    run.push(RecoveredTable {
                        id,
                        global_seqno: reader.read_u64::<LittleEndian>()?,
                    });
                }

                level.push(run);
            }

            levels.push(level);
        }

        if !reader.is_empty() {
            return Err(Error::Unrecoverable);
        }

        Ok(Self {
            curr_version_id,
            table_ids: levels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::WriteBytesExt;
    use test_log::test;

    fn append_checksum(current: &mut Vec<u8>) -> Result<()> {
        let checksum = xxhash_rust::xxh3::xxh3_128(current);
        current.write_u128::<LittleEndian>(checksum)?;
        Ok(())
    }

    #[test]
    fn recovery_reads_table_entry() -> Result<()> {
        const TABLE_ID: u32 = 42;
        const GLOBAL_SEQNO: u64 = 84;

        let directory = tempfile::tempdir()?;
        let mut current = CURRENT_MAGIC.to_vec();
        current.write_u64::<LittleEndian>(7)?;

        for level in 0..DEFAULT_LEVEL_COUNT {
            if level == 0 {
                current.write_u8(1)?;
                current.write_u32::<LittleEndian>(1)?;
                current.write_u32::<LittleEndian>(TABLE_ID)?;
                current.write_u64::<LittleEndian>(GLOBAL_SEQNO)?;
            } else {
                current.write_u8(0)?;
            }
        }

        append_checksum(&mut current)?;

        fs::write(directory.path().join(CURRENT_VERSION_FILE), current)?;

        let recovery = Recovery::load(directory.path())?;
        let Some(table) = recovery
            .table_ids
            .first()
            .and_then(|level| level.first())
            .and_then(|run| run.first())
        else {
            panic!("recovered table should exist");
        };
        assert_eq!(TABLE_ID, table.id);
        assert_eq!(GLOBAL_SEQNO, table.global_seqno);

        Ok(())
    }

    #[test]
    fn recovery_reads_checksumless_v9_manifest() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let mut current = CHECKSUMLESS_CURRENT_MAGIC.to_vec();
        current.write_u64::<LittleEndian>(7)?;
        for _ in 0..DEFAULT_LEVEL_COUNT {
            current.write_u8(0)?;
        }
        fs::write(directory.path().join(CURRENT_VERSION_FILE), current)?;

        let recovery = Recovery::load(directory.path())?;
        assert_eq!(7, recovery.curr_version_id);
        assert!(recovery.table_ids.iter().all(Vec::is_empty));
        Ok(())
    }

    #[test]
    fn recovery_rejects_manifest_checksum_mismatch() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let version = crate::version::Version::new(7);
        version.persist(directory.path())?;

        let path = directory.path().join(CURRENT_VERSION_FILE);
        let mut current = fs::read(&path)?;
        let payload_byte = CURRENT_MAGIC.len() + size_of::<u64>();
        *current.get_mut(payload_byte).ok_or(Error::Unrecoverable)? ^= 1;
        fs::write(path, current)?;

        assert!(matches!(
            Recovery::load(directory.path()),
            Err(Error::Unrecoverable)
        ));
        Ok(())
    }

    #[test]
    fn recovery_rejects_impossible_table_count_before_allocating() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let mut current = CURRENT_MAGIC.to_vec();
        current.write_u64::<LittleEndian>(7)?;
        current.write_u8(1)?;
        current.write_u32::<LittleEndian>(u32::MAX)?;
        for _ in 1..DEFAULT_LEVEL_COUNT {
            current.write_u8(0)?;
        }
        append_checksum(&mut current)?;
        fs::write(directory.path().join(CURRENT_VERSION_FILE), current)?;

        assert!(matches!(
            Recovery::load(directory.path()),
            Err(Error::Unrecoverable)
        ));
        Ok(())
    }
}
