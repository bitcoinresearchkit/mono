use std::{
    fs::{self, File},
    io,
    ops::Range,
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
};

use bitview_cohort::{AgeRange, AgeRangeId, UTXOAggregateId};
use brk_error::{Error, Result};
use brk_types::{CentsCompact, Date, Sats, UrpdRaw};
use vecdb::ColumnId;

use super::{AgeRangeUrpds, DIR_NAME, HEADER_LEN};

impl AgeRangeUrpds {
    pub fn dir(states_path: &Path) -> PathBuf {
        states_path.join(DIR_NAME)
    }

    pub fn path(states_path: &Path, date: Date) -> PathBuf {
        Self::dir(states_path).join(date.to_string())
    }

    pub fn read(states_path: &Path, date: Date) -> Result<Self> {
        let path = Self::path(states_path, date);
        let data = fs::read(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Cannot read age-range URPD '{}': {error}", path.display()),
            )
        })?;
        let ranges = Self::ranges(&data, data.len())?;
        let entries = AgeRange::try_from_fn(|id| {
            UrpdRaw::deserialize_entries(&data[id.select(&ranges).clone()])
        })?;
        Ok(Self { entries })
    }

    pub fn read_one(states_path: &Path, id: AgeRangeId, date: Date) -> Result<UrpdRaw> {
        let path = Self::path(states_path, date);
        let (file, ranges) = Self::open(&path)?;
        let range = id.select(&ranges);
        let mut data = vec![0; range.len()];
        file.read_exact_at(&mut data, range.start as u64)?;
        Ok(UrpdRaw {
            map: UrpdRaw::deserialize_entries(&data)?.into_iter().collect(),
        })
    }

    pub fn read_aggregate(states_path: &Path, id: UTXOAggregateId, date: Date) -> Result<UrpdRaw> {
        if id == UTXOAggregateId::All {
            return Ok(Self::read(states_path, date)?.aggregate(id));
        }

        let path = Self::path(states_path, date);
        let (file, ranges) = Self::open(&path)?;
        let ids = id.age_range_ids();
        debug_assert!(
            ids.windows(2)
                .all(|pair| pair[0].index() + 1 == pair[1].index())
        );
        let first = ids.first().expect("aggregate contains an age range");
        let last = ids.last().expect("aggregate contains an age range");
        let start = first.select(&ranges).start;
        let end = last.select(&ranges).end;
        let mut data = vec![0; end - start];
        file.read_exact_at(&mut data, start as u64)?;

        let entries = ids.iter().try_fold(Vec::new(), |left, id| {
            let range = id.select(&ranges);
            let section = &data[range.start - start..range.end - start];
            let right = UrpdRaw::deserialize_entries(section)?;
            Ok::<_, Error>(Self::merge_sorted(&left, &right))
        })?;
        Ok(UrpdRaw {
            map: entries.into_iter().collect(),
        })
    }

    fn open(path: &Path) -> Result<(File, AgeRange<Range<usize>>)> {
        let file = File::open(path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Cannot read age-range URPD '{}': {error}", path.display()),
            )
        })?;
        let mut header = [0; HEADER_LEN];
        file.read_exact_at(&mut header, 0)?;
        let ranges = Self::ranges(&header, file.metadata()?.len() as usize)?;
        Ok((file, ranges))
    }

    pub fn get(&self, id: AgeRangeId) -> &[(CentsCompact, Sats)] {
        id.select(&self.entries)
    }
}
