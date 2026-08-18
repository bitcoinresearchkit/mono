mod recovered_table;

pub use recovered_table::RecoveredTable;

use crate::{
    Checksum,
    file::{CURRENT_MAGIC, CURRENT_VERSION_FILE},
    version::{DEFAULT_LEVEL_COUNT, VersionId},
};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::path::Path;

pub struct Recovery {
    pub curr_version_id: VersionId,
    pub table_ids: Vec<Vec<Vec<RecoveredTable>>>,
}

impl Recovery {
    pub fn load(folder: &Path) -> crate::Result<Self> {
        let current_path = folder.join(CURRENT_VERSION_FILE);
        let bytes = std::fs::read(&current_path)?;
        let magic = bytes
            .get(..CURRENT_MAGIC.len())
            .ok_or(crate::Error::Unrecoverable)?;
        if magic != CURRENT_MAGIC {
            let version = magic.last().copied().ok_or(crate::Error::Unrecoverable)?;
            return Err(crate::Error::InvalidVersion(version));
        }
        if bytes.len() < CURRENT_MAGIC.len() + size_of::<VersionId>() + size_of::<u128>() {
            return Err(crate::Error::Unrecoverable);
        }
        let (payload, checksum) = bytes.split_at(bytes.len() - size_of::<u128>());
        Checksum::from_raw(xxhash_rust::xxh3::xxh3_128(payload))
            .check(Checksum::from_raw(LittleEndian::read_u128(checksum)))?;
        let mut reader = payload
            .get(CURRENT_MAGIC.len()..)
            .ok_or(crate::Error::Unrecoverable)?;
        let curr_version_id = reader.read_u64::<LittleEndian>()?;

        log::info!("Recovering current manifest at {}", current_path.display());
        let mut levels = Vec::with_capacity(usize::from(DEFAULT_LEVEL_COUNT));

        for _ in 0..DEFAULT_LEVEL_COUNT {
            let mut level = Vec::new();
            let run_count = reader.read_u8()?;

            for _ in 0..run_count {
                let table_count = reader.read_u32::<LittleEndian>()?;
                let capacity =
                    usize::try_from(table_count).map_err(|_| crate::Error::Unrecoverable)?;
                let mut run = Vec::with_capacity(capacity);

                for _ in 0..table_count {
                    run.push(RecoveredTable {
                        id: reader.read_u32::<LittleEndian>()?,
                        checksum: Checksum::from_raw(reader.read_u128::<LittleEndian>()?),
                        global_seqno: reader.read_u64::<LittleEndian>()?,
                    });
                }

                level.push(run);
            }

            levels.push(level);
        }

        if !reader.is_empty() {
            return Err(crate::Error::Unrecoverable);
        }

        Ok(Self {
            curr_version_id,
            table_ids: levels,
        })
    }
}
