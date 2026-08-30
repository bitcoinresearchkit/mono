mod recovered_table;

pub use recovered_table::RecoveredTable;

use crate::{
    Error, Result,
    file::{CURRENT_MAGIC, CURRENT_VERSION_FILE},
    version::DEFAULT_LEVEL_COUNT,
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{fs, path::Path};

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
        if magic != CURRENT_MAGIC {
            let version = magic.last().copied().ok_or(Error::Unrecoverable)?;
            return Err(Error::InvalidVersion(version));
        }
        if bytes.len() < CURRENT_MAGIC.len() + size_of::<u64>() {
            return Err(Error::Unrecoverable);
        }
        let mut reader = bytes
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
}
