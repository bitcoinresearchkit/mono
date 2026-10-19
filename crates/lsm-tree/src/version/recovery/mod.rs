use std::{fs, path::Path};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use log::{error, info};
use xxhash_rust::xxh3;

use crate::{
    Error, Result,
    file::{CURRENT_VERSION_FILE, MAGIC_BYTES},
    version::DEFAULT_LEVEL_COUNT,
};

mod recovered_table;

pub use recovered_table::RecoveredTable;

const TABLE_ENTRY_SIZE: usize = size_of::<u32>() + size_of::<u64>();

pub struct Recovery {
    pub curr_version_id: u64,
    pub table_ids: Vec<Vec<Vec<RecoveredTable>>>,
}

impl Recovery {
    pub fn load(folder: &Path) -> Result<Self> {
        let current_path = folder.join(CURRENT_VERSION_FILE);
        let bytes = fs::read(&current_path)?;
        let magic = bytes.get(..MAGIC_BYTES.len()).ok_or(Error::Unrecoverable)?;
        if magic != MAGIC_BYTES {
            let version = magic.last().copied().ok_or(Error::Unrecoverable)?;
            return Err(Error::InvalidVersion(version));
        }
        if bytes.len() < MAGIC_BYTES.len() + size_of::<u64>() + size_of::<u128>() {
            return Err(Error::Unrecoverable);
        }
        let (payload, checksum) = bytes.split_at(bytes.len() - size_of::<u128>());
        if xxh3::xxh3_128(payload) != LittleEndian::read_u128(checksum) {
            error!("Current manifest checksum mismatch");
            return Err(Error::Unrecoverable);
        }

        let mut reader = payload
            .get(MAGIC_BYTES.len()..)
            .ok_or(Error::Unrecoverable)?;
        let curr_version_id = reader.read_u64::<LittleEndian>()?;

        info!("Recovering current manifest at {}", current_path.display());
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
