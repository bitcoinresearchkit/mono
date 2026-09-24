use std::{
    fs::File,
    io,
    ops::Range,
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
};

use bitview_cohort::{AgeRange, AgeRangeId};
use brk_error::Result;
use brk_types::Date;

use super::{AgeRangeUrpds, format::HEADER_LEN};
use crate::UrpdRaw;

const DIR_NAME: &str = "utxos_age_range_urpds";

impl AgeRangeUrpds {
    pub fn dir(states_path: &Path) -> PathBuf {
        states_path.join(DIR_NAME)
    }

    pub fn path(states_path: &Path, date: Date) -> PathBuf {
        Self::dir(states_path).join(date.to_string())
    }

    pub fn read(states_path: &Path, date: Date) -> Result<Self> {
        let data = Self::read_bytes(states_path, date)?;
        let ranges = Self::ranges(&data, data.len())?;
        let entries = AgeRange::try_from_fn(|id| {
            UrpdRaw::deserialize_entries(&data[id.select(&ranges).clone()])
        })?;
        Ok(Self { entries })
    }

    pub(super) fn read_bytes(states_path: &Path, date: Date) -> Result<Vec<u8>> {
        let path = Self::path(states_path, date);
        let data = UrpdRaw::read_encoded_file(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Cannot read age-range URPD '{}': {error}", path.display()),
            )
        })?;
        Ok(data)
    }

    /// Read one encoded section without decompressing it or loading other cohorts.
    /// Callers must hold the producer's publication guard during the file read.
    pub fn read_one_bytes(states_path: &Path, id: AgeRangeId, date: Date) -> Result<Vec<u8>> {
        let path = Self::path(states_path, date);
        let (file, ranges) = Self::open(&path)?;
        let range = id.select(&ranges);
        let mut data = vec![0; range.len()];
        file.read_exact_at(&mut data, range.start as u64)?;
        Ok(data)
    }

    pub(super) fn open(path: &Path) -> Result<(File, AgeRange<Range<usize>>)> {
        let file = File::open(path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Cannot read age-range URPD '{}': {error}", path.display()),
            )
        })?;
        let mut header = [0; HEADER_LEN];
        let file_len = file.metadata()?.len();
        if file_len > UrpdRaw::MAX_ENCODED_BYTES as u64 {
            return Err(Self::invalid("file exceeds snapshot limit"));
        }
        file.read_exact_at(&mut header, 0)?;
        let ranges = Self::ranges(&header, file_len as usize)?;
        Ok((file, ranges))
    }
}
