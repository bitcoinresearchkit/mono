use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use brk_error::Result;
use brk_types::{CentsCompact, Date, Sats};

use crate::UrpdRaw;

impl UrpdRaw {
    pub fn dir(states_path: &Path, name: &str) -> PathBuf {
        states_path.join(name).join("urpd")
    }

    pub fn path(states_path: &Path, name: &str, date: Date) -> PathBuf {
        Self::dir(states_path, name).join(date.to_string())
    }

    /// Capture encoded input while holding the producer's publication guard.
    /// The owned bytes can be decoded after releasing the guard.
    pub fn read_bytes(states_path: &Path, name: &str, date: Date) -> Result<Vec<u8>> {
        let path = Self::path(states_path, name, date);
        Self::read_encoded_file(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Cannot read URPD '{}': {error}", path.display()),
            )
            .into()
        })
    }

    /// Read a regular snapshot file without exceeding the encoded size limit.
    pub fn read_encoded_file(path: &Path) -> io::Result<Vec<u8>> {
        let file = fs::File::open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() > Self::MAX_ENCODED_BYTES as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "URPD file exceeds snapshot limits",
            ));
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize + 1);
        file.take(Self::MAX_ENCODED_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > Self::MAX_ENCODED_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "URPD file grew beyond snapshot limits",
            ));
        }
        Ok(bytes)
    }

    /// Publish sorted entries while holding the producer's publication guard.
    pub fn write(
        states_path: &Path,
        name: &str,
        date: Date,
        entries: impl Iterator<Item = (CentsCompact, Sats)>,
    ) -> Result<()> {
        let dir = Self::dir(states_path, name);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join(date.to_string()), Self::serialize_iter(entries)?)?;
        Ok(())
    }
}
