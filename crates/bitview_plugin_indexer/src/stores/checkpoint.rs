use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use bitview_primitives::Lengths;
use brk_error::{Error, Result};
use brk_exit::Exit;

/// The stores' checkpoint: the lengths of the indexed data they hold, which a start brings the
/// vectors back to. It is never removed. A commit writes the vectors, the stores, then this; a
/// rollback writes this, undoes the stores, then truncates the vectors. Whatever stops either
/// midway leaves the stores between this and the vectors, and the vectors holding every row the
/// stores may hold.
#[derive(Debug, Clone)]
pub struct StoresCheckpoint {
    path: PathBuf,
}

impl StoresCheckpoint {
    pub fn new(stores_path: &Path) -> Self {
        Self {
            path: stores_path.join("lengths"),
        }
    }

    /// `None` before the first write.
    pub fn read(&self) -> Result<Option<Lengths>> {
        match fs::read(&self.path) {
            Ok(bytes) => decode(&bytes)
                .map(Some)
                .ok_or(Error::InvalidData("Indexer stores checkpoint is malformed")),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn write(&self, lengths: &Lengths) -> Result<()> {
        debug_assert!(Exit::is_locked(), "stores checkpoint written outside the exit lock");
        let pending = self.path.with_extension("pending");
        fs::write(&pending, encode(lengths))?;
        fs::rename(&pending, &self.path)?;
        Ok(())
    }
}

/// The file: every length as a little-endian u64, in this order.
macro_rules! lengths_file {
    ($($field:ident),+ $(,)?) => {
        const FILE_LEN: usize = [$(stringify!($field)),+].len() * size_of::<u64>();

        fn encode(lengths: &Lengths) -> Vec<u8> {
            [$(usize::from(lengths.$field) as u64),+]
                .into_iter()
                .flat_map(u64::to_le_bytes)
                .collect()
        }

        fn decode(bytes: &[u8]) -> Option<Lengths> {
            if bytes.len() != FILE_LEN {
                return None;
            }
            let mut values = bytes
                .chunks_exact(size_of::<u64>())
                .map(|value| u64::from_le_bytes(value.try_into().unwrap()) as usize);
            Some(Lengths { $($field: values.next()?.into()),+ })
        }
    };
}

lengths_file!(
    height,
    tx_index,
    txin_index,
    txout_index,
    empty_script_output_index,
    op_return_output_index,
    p2ms_output_index,
    unknown_script_output_index,
    p2pk65_addr_index,
    p2pk33_addr_index,
    p2pkh_addr_index,
    p2sh_addr_index,
    p2wpkh_addr_index,
    p2wsh_addr_index,
    p2tr_addr_index,
    p2a_addr_index,
);
