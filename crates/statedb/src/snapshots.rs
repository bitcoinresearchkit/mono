use crate::{
    State,
    snapshot::{self, MAX_RECORD, Snapshot},
    util::invalid,
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Error, Result},
    os::unix::fs::FileExt,
    path::Path,
};
const MAGIC: &[u8; 8] = b"STAPAGE4";
const HEADER: usize = 9;
const ROW: usize = 12;
/// Full snapshot pages with one replaceable tip. The caller holds exit and publication guards.
pub(crate) struct Snapshots {
    data: File,
    pages: File,
    writable: bool,
    entries: Vec<(usize, u64, u64)>,
    retained: usize,
    end: u64,
    writing: bool,
}
impl Snapshots {
    pub fn open(path: &Path, writable: bool) -> Result<Self> {
        if writable {
            fs::create_dir_all(path)?;
        }
        let open = |name| {
            OpenOptions::new()
                .create(writable)
                .truncate(false)
                .read(true)
                .write(writable)
                .open(path.join(name))
        };
        let data = open("data")?;
        if writable {
            data.try_lock().map_err(Error::other)?;
        }
        let pages = open("pages")?;
        let size = pages.metadata()?.len();
        if size == 0 && writable {
            if data.metadata()?.len() != 0 {
                return Err(invalid("missing snapshot page index"));
            }
            pages.write_all_at(&[MAGIC.as_slice(), &[0]].concat(), 0)?;
            pages.sync_data()?;
        }
        let size = pages.metadata()?.len();
        if size < HEADER as u64
            || size > MAX_RECORD
            || !(size as usize - HEADER).is_multiple_of(ROW)
        {
            return Err(invalid("invalid snapshot page index length"));
        }
        let mut bytes = vec![0; size as usize];
        pages.read_exact_at(&mut bytes, 0)?;
        if &bytes[..8] != MAGIC {
            return Err(invalid("unsupported snapshot page index"));
        }
        let retained_last = bytes[8];
        if retained_last > 1 || bytes.len() == HEADER && retained_last != 0 {
            return Err(invalid("invalid snapshot retention"));
        }
        let mut entries: Vec<(usize, u64, u64)> = Vec::with_capacity((bytes.len() - HEADER) / ROW);
        let mut end = 0;
        for row in bytes[HEADER..].chunks_exact(ROW) {
            let h = u32::from_le_bytes(row[..4].try_into().unwrap()) as usize;
            let next = u64::from_le_bytes(row[4..].try_into().unwrap());
            if h > 100_000_000
                || entries.last().is_some_and(|&(prev, _, _)| prev >= h)
                || next < end
                || !(72..=MAX_RECORD).contains(&(next - end))
            {
                return Err(invalid("invalid snapshot page entry"));
            }
            entries.push((h, end, next - end));
            end = next;
        }
        if end != data.metadata()?.len() {
            return Err(invalid("snapshot page index does not cover data"));
        }
        let retained = entries
            .len()
            .saturating_sub(usize::from(retained_last == 0));
        Ok(Self {
            data,
            pages,
            writable,
            entries,
            retained,
            end,
            writing: false,
        })
    }
    fn healthy(&self) -> Result<()> {
        if self.writing {
            Err(invalid(
                "snapshot update did not finish; reopen before reuse",
            ))
        } else {
            Ok(())
        }
    }
    fn read(&self, entry: (usize, u64, u64), versions: (u64, u64)) -> Result<Option<Snapshot>> {
        self.healthy()?;
        let (h, at, n) = entry;
        let snapshot = Snapshot::open(&self.data, at, n as usize, versions)?;
        if snapshot.as_ref().is_some_and(|s| s.len != h) {
            return Err(invalid("snapshot page height mismatch"));
        }
        Ok(snapshot)
    }
    pub fn latest(&self, versions: (u64, u64)) -> Result<Option<Snapshot>> {
        self.healthy()?;
        self.entries
            .last()
            .map_or(Ok(None), |&e| self.read(e, versions))
    }
    pub fn before(
        &self,
        end: usize,
        versions: (u64, u64),
    ) -> impl Iterator<Item = Result<Option<Snapshot>>> + '_ {
        let count = self.entries[..self.retained].partition_point(|&(h, _, _)| h <= end);
        self.entries[..count]
            .iter()
            .rev()
            .map(move |&e| self.read(e, versions))
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn begin(&mut self, start: usize, include_start: bool) -> Result<()> {
        self.healthy()?;
        if !self.writable {
            return Err(invalid("read-only snapshots"));
        }
        self.writing = true;
        let count = self.entries[..self.retained]
            .partition_point(|&(h, _, _)| h < start || include_start && h == start);
        let end = self
            .entries
            .get(count.wrapping_sub(1))
            .map_or(0, |&(_, at, n)| at + n);
        self.data.set_len(end)?;
        self.entries.truncate(count);
        self.retained = count;
        self.end = end;
        Ok(())
    }
    pub fn push(&mut self, state: &State, versions: (u64, u64), retained: bool) -> Result<()> {
        if !self.writing || !self.writable {
            return Err(invalid("snapshot write outside update"));
        }
        if self.retained != self.entries.len()
            || state.len() > 100_000_000
            || self
                .entries
                .last()
                .is_some_and(|&(h, _, _)| h >= state.len())
        {
            return Err(invalid("invalid snapshot append"));
        }
        let n = snapshot::write(&self.data, self.end, state, versions, retained)?;
        self.entries.push((state.len(), self.end, n));
        self.end += n;
        if retained {
            self.retained = self.entries.len();
        }
        Ok(())
    }
    pub fn finish(&mut self) -> Result<()> {
        if !self.writing || !self.writable {
            return Err(invalid("snapshot commit outside update"));
        }
        self.data.sync_data()?;
        let mut index = Vec::with_capacity(HEADER + self.entries.len() * ROW);
        index.extend_from_slice(MAGIC);
        index.push(u8::from(
            !self.entries.is_empty() && self.retained == self.entries.len(),
        ));
        for &(h, at, n) in &self.entries {
            index.extend_from_slice(&(h as u32).to_le_bytes());
            index.extend_from_slice(&(at + n).to_le_bytes());
        }
        self.pages.write_all_at(&index, 0)?;
        self.pages.set_len(index.len() as u64)?;
        self.pages.sync_data()?;
        self.writing = false;
        Ok(())
    }
}
