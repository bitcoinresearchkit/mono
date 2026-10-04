use crate::util::{atomic_write, invalid};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Error, Read, Result, Seek, SeekFrom, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
};

const MAGIC: &[u8; 8] = b"ORIGIN03";
const INDEX_BYTES: usize = 8;
const MAX_RECORD: usize = 128 * 1024 * 1024;

/// A single writer; readers borrow the published prefix. Unpublished tails are discarded on reopen.
pub(crate) struct Journal {
    path: PathBuf,
    _lock: Option<File>,
    read_end: u64,
    data: BufWriter<File>,
    index_file: File,
    index: Vec<u64>,
    committed: usize,
    poisoned: bool,
    pub version: u64,
}
impl Journal {
    pub fn open(path: &Path) -> Result<Self> {
        Self::open_inner(path, true)
    }
    pub fn open_reader(path: &Path) -> Result<Self> {
        Self::open_inner(path, false)
    }
    fn open_inner(path: &Path, writable: bool) -> Result<Self> {
        if writable {
            fs::create_dir_all(path)?;
        }
        let lock = if writable {
            Some(
                OpenOptions::new()
                    .create(true)
                    .truncate(false)
                    .read(true)
                    .write(true)
                    .open(path.join("writer"))?,
            )
        } else {
            None
        };
        if let Some(lock) = &lock {
            lock.try_lock().map_err(Error::other)?;
        }
        let mut data = OpenOptions::new()
            .create(writable)
            .truncate(false)
            .read(true)
            .write(writable)
            .open(path.join("data"))?;
        let mut index_file = OpenOptions::new()
            .create(writable)
            .truncate(false)
            .read(true)
            .write(writable)
            .open(path.join("index"))?;
        let manifest = path.join("commit");
        // Manifest words: version, start height (always 0, kept for the format), count, data end.
        let (version, count, end) = if manifest.try_exists()? {
            let bytes = fs::read(&manifest)?;
            if bytes.len() != 40 || &bytes[..8] != MAGIC {
                return Err(invalid("invalid origin journal manifest"));
            }
            let get = |i| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
            if get(16) != 0 {
                return Err(invalid("origin journal starts above genesis"));
            }
            (get(8), get(24) as usize, get(32))
        } else {
            if !writable {
                return Err(invalid("missing published journal"));
            }
            if data.metadata()?.len() != 0 || index_file.metadata()?.len() != 0 {
                return Err(invalid("missing commit for nonempty origin journal"));
            }
            (0, 0, 0)
        };
        let index_len = count
            .checked_mul(INDEX_BYTES)
            .ok_or_else(|| invalid("index overflow"))?;
        if count > u32::MAX as usize
            || index_file.metadata()?.len() < index_len as u64
            || data.metadata()?.len() < end
        {
            return Err(invalid("incomplete committed journal"));
        }
        let mut index = Vec::new();
        if writable {
            let mut bytes = vec![0; index_len];
            index_file.read_exact(&mut bytes)?;
            index.reserve(count);
            let mut previous = 0;
            for chunk in bytes.as_chunks::<INDEX_BYTES>().0 {
                let next = u64::from_le_bytes(chunk[..8].try_into().unwrap());
                if next < previous || next - previous > MAX_RECORD as u64 {
                    return Err(invalid("invalid journal offset"));
                }
                index.push(next);
                previous = next;
            }
            if previous != end {
                return Err(invalid("journal index does not match commit"));
            }
            data.set_len(end)?;
            data.seek(SeekFrom::Start(end))?;
            index_file.set_len(index_len as u64)?;
            index_file.seek(SeekFrom::Start(index_len as u64))?;
        } else if count > 0 {
            let mut last = [0; INDEX_BYTES];
            index_file.read_exact_at(&mut last, ((count - 1) * INDEX_BYTES) as u64)?;
            if u64::from_le_bytes(last[..8].try_into().unwrap()) != end {
                return Err(invalid("journal index does not match commit"));
            }
        } else if end != 0 {
            return Err(invalid("empty journal has data"));
        }
        let mut this = Self {
            path: path.to_owned(),
            _lock: lock,
            read_end: end,
            data: BufWriter::with_capacity(if writable { 1024 * 1024 } else { 0 }, data),
            index_file,
            index,
            committed: count,
            poisoned: false,
            version,
        };
        if !manifest.try_exists()? {
            this.commit_inner(true)?;
        }
        Ok(this)
    }
    pub fn len(&self) -> usize {
        if self._lock.is_some() {
            self.index.len()
        } else {
            self.committed
        }
    }
    fn healthy(&self) -> Result<()> {
        if self.poisoned {
            Err(invalid("origin writer failed; reopen before reuse"))
        } else {
            Ok(())
        }
    }
    pub fn push(&mut self, bytes: &[u8]) -> Result<()> {
        if self._lock.is_none() {
            return Err(invalid("read-only journal"));
        }
        self.healthy()?;
        if bytes.len() > MAX_RECORD || self.len() >= u32::MAX as usize {
            return Err(invalid("origin record exceeds limits"));
        }
        let end = self.index.last().map_or(0, |&v| v) + bytes.len() as u64;
        if let Err(error) = self.data.write_all(bytes) {
            self.poisoned = true;
            return Err(error);
        }
        self.index.push(end);
        Ok(())
    }
    fn bounds(&self, height: usize) -> Result<(u64, u64)> {
        self.healthy()?;
        if height >= self.committed {
            return Err(invalid("height outside published origin history"));
        }
        let (start, end) = if self._lock.is_some() {
            (
                if height == 0 {
                    0
                } else {
                    self.index[height - 1]
                },
                self.index[height],
            )
        } else {
            let mut offsets = [0; 2 * INDEX_BYTES];
            let first = height.saturating_sub(1);
            let n = if height == 0 {
                INDEX_BYTES
            } else {
                2 * INDEX_BYTES
            };
            self.index_file
                .read_exact_at(&mut offsets[..n], (first * INDEX_BYTES) as u64)?;
            (
                if height == 0 {
                    0
                } else {
                    u64::from_le_bytes(offsets[..8].try_into().unwrap())
                },
                u64::from_le_bytes(offsets[n - 8..n].try_into().unwrap()),
            )
        };
        if end < start
            || end - start > MAX_RECORD as u64
            || self._lock.is_none() && end > self.read_end
        {
            return Err(invalid("invalid published journal offset"));
        }
        Ok((start, end))
    }
    pub(crate) fn read_prefix(&self, height: usize, bytes: &mut [u8]) -> Result<()> {
        let (start, end) = self.bounds(height)?;
        if end - start < bytes.len() as u64 {
            return Err(invalid("truncated record prefix"));
        }
        self.data.get_ref().read_exact_at(bytes, start)
    }
    pub(crate) fn read_batch(
        &self,
        height: usize,
        end: usize,
        index_bytes: &mut Vec<u8>,
        records: &mut Vec<usize>,
        bytes: &mut Vec<u8>,
    ) -> Result<()> {
        self.healthy()?;
        if height >= end || end > self.committed {
            return Err(invalid("range outside published origin history"));
        }
        let n = end - height;
        let first = height.saturating_sub(1);
        if self._lock.is_none() {
            index_bytes.resize((n + usize::from(height > 0)) * INDEX_BYTES, 0);
            self.index_file
                .read_exact_at(index_bytes, (first * INDEX_BYTES) as u64)?;
        }
        let offset = |entry: usize| {
            if self._lock.is_some() {
                self.index[entry]
            } else {
                let at = (entry - first) * INDEX_BYTES;
                let v = &index_bytes[at..at + INDEX_BYTES];
                u64::from_le_bytes(v.try_into().unwrap())
            }
        };
        let start = if height == 0 { 0 } else { offset(height - 1) };
        let published_end = if self._lock.is_some() {
            self.index[self.committed - 1]
        } else {
            self.read_end
        };
        let mut previous = start;
        records.clear();
        for entry in height..end {
            let next = offset(entry);
            if next < previous || next > published_end || next - previous > MAX_RECORD as u64 {
                return Err(invalid("invalid published journal offset"));
            }
            // A large single record remains readable; ordinary windows stay bounded.
            if next - start > 1024 * 1024 && !records.is_empty() {
                break;
            }
            records.push((next - start) as usize);
            previous = next;
        }
        bytes.resize((previous - start) as usize, 0);
        self.data.get_ref().read_exact_at(bytes, start)?;
        Ok(())
    }
    pub fn commit(&mut self) -> Result<()> {
        self.commit_inner(false)
    }
    fn commit_inner(&mut self, force: bool) -> Result<()> {
        if self._lock.is_none() {
            return Err(invalid("read-only journal"));
        }
        self.healthy()?;
        if !force && self.index.len() == self.committed {
            return Ok(());
        }
        let result = (|| {
            self.data.flush()?;
            self.data.get_ref().sync_data()?;
            let mut bytes = Vec::with_capacity((self.index.len() - self.committed) * INDEX_BYTES);
            for &end in &self.index[self.committed..] {
                bytes.extend_from_slice(&end.to_le_bytes());
            }
            self.index_file
                .seek(SeekFrom::Start((self.committed * INDEX_BYTES) as u64))?;
            self.index_file.write_all(&bytes)?;
            self.index_file.sync_data()?;
            self.publish(self.index.len())?;
            self.committed = self.index.len();
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn publish(&self, count: usize) -> Result<()> {
        let mut bytes = MAGIC.to_vec();
        for n in [
            self.version,
            0,
            count as u64,
            if count == 0 { 0 } else { self.index[count - 1] },
        ] {
            bytes.extend_from_slice(&n.to_le_bytes());
        }
        atomic_write(&self.path.join("commit"), &bytes)
    }
    pub fn truncate(&mut self, height: usize) -> Result<()> {
        if self._lock.is_none() {
            return Err(invalid("read-only journal"));
        }
        self.healthy()?;
        if height > self.index.len() {
            return Err(invalid("invalid origin truncate"));
        }
        let n = height;
        if n > self.committed {
            self.commit()?;
        }
        let result = (|| {
            self.data.flush()?;
            self.publish(n)?;
            let end = if n == 0 { 0 } else { self.index[n - 1] };
            self.data.get_mut().set_len(end)?;
            self.data.get_mut().seek(SeekFrom::Start(end))?;
            self.index_file.set_len((n * INDEX_BYTES) as u64)?;
            self.index_file
                .seek(SeekFrom::Start((n * INDEX_BYTES) as u64))?;
            self.index.truncate(n);
            self.committed = n;
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub fn validate_version(&mut self, version: u64) -> Result<()> {
        if self._lock.is_none() {
            return Err(invalid("read-only journal"));
        }
        self.healthy()?;
        if version != self.version {
            self.truncate(0)?;
            self.version = version;
            self.commit_inner(true)?;
        }
        Ok(())
    }
}
