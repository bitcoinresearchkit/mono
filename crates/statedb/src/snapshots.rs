use crate::{
    State,
    snapshot::{self, MAX_RECORD, Snapshot},
    util::{atomic_write, invalid},
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};
const MAGIC: &[u8; 8] = b"STAPAGE4";
const HEADER: usize = 9;
const ROW: usize = 12;
const TIP: &str = "tip";
const PREV: &str = "prev";
const NEXT: &str = "tip.next";
/// Full snapshots, published so readers never see one change: retained snapshots are appended to
/// `data` and listed by `pages` (replaced by rename); the newest non-retained snapshot is the `tip`
/// file and the one it replaced is `prev` (both replaced by rename), so a reader still at the
/// previous publication restores from a snapshot too. Only a rewind truncates retained snapshots.
pub(crate) struct Snapshots {
    path: PathBuf,
    data: File,
    writable: bool,
    entries: Vec<(usize, u64, u64)>,
    end: u64,
    /// `tip` then `prev`, as opened.
    tips: Vec<File>,
    pages_changed: bool,
    tip_pushed: bool,
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
        let pages_path = path.join("pages");
        let mut bytes = fs::read(&pages_path).or_else(|error| match error.kind() {
            ErrorKind::NotFound if writable => Ok(Vec::new()),
            _ => Err(error),
        })?;
        if bytes.is_empty() && writable {
            if data.metadata()?.len() != 0 {
                return Err(invalid("missing snapshot page index"));
            }
            bytes = [MAGIC.as_slice(), &[0]].concat();
            atomic_write(&pages_path, &bytes)?;
        }
        if bytes.len() < HEADER
            || bytes.len() as u64 > MAX_RECORD
            || !(bytes.len() - HEADER).is_multiple_of(ROW)
        {
            return Err(invalid("invalid snapshot page index length"));
        }
        if &bytes[..8] != MAGIC {
            return Err(invalid("unsupported snapshot page index"));
        }
        let retained_last = bytes[8];
        if retained_last > 1 || bytes.len() == HEADER && retained_last != 0 {
            return Err(invalid("invalid snapshot retention"));
        }
        let mut entries: Vec<(usize, u64, u64)> = Vec::with_capacity((bytes.len() - HEADER) / ROW);
        let mut end = 0;
        for row in bytes[HEADER..].as_chunks::<ROW>().0 {
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
        // Older layouts kept the replaceable tip as the last entry of `data`: drop it.
        if retained_last == 0
            && let Some((_, at, _)) = entries.pop()
        {
            end = at;
        }
        // A reader may see snapshots appended after its index, or a rewind's truncation of
        // entries its index still lists (rewinds shorten the index first); a writer drops both.
        let len = data.metadata()?.len();
        if end > len {
            if writable {
                return Err(invalid("snapshot page index exceeds data"));
            }
            while entries.last().is_some_and(|&(_, at, n)| at + n > len) {
                entries.pop();
            }
            end = entries.last().map_or(0, |&(_, at, n)| at + n);
        }
        let mut this = Self {
            path: path.to_owned(),
            data,
            writable,
            entries,
            end,
            tips: Vec::new(),
            pages_changed: false,
            tip_pushed: false,
            writing: false,
        };
        if writable {
            if len != end {
                this.data.set_len(end)?;
                atomic_write(&pages_path, &this.index())?;
            }
            remove_if_exists(&path.join(NEXT))?;
        }
        this.tips = this.open_tips()?;
        Ok(this)
    }
    fn open_tips(&self) -> Result<Vec<File>> {
        let mut tips = Vec::with_capacity(2);
        for name in [TIP, PREV] {
            match File::open(self.path.join(name)) {
                Ok(file) => tips.push(file),
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(tips)
    }
    fn index(&self) -> Vec<u8> {
        let mut index = Vec::with_capacity(HEADER + self.entries.len() * ROW);
        index.extend_from_slice(MAGIC);
        index.push(u8::from(!self.entries.is_empty()));
        for &(h, at, n) in &self.entries {
            index.extend_from_slice(&(h as u32).to_le_bytes());
            index.extend_from_slice(&(at + n).to_le_bytes());
        }
        index
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
    /// The newest snapshot: a tip, or the last retained snapshot.
    pub fn latest(&self, versions: (u64, u64)) -> Result<Option<Snapshot>> {
        let tip = self.tips(versions)?.into_iter().next();
        let retained = self
            .entries
            .last()
            .map_or(Ok(None), |&e| self.read(e, versions))?;
        Ok(match (tip, retained) {
            (Some(tip), Some(retained)) if retained.len > tip.len => Some(retained),
            (tip, retained) => tip.or(retained),
        })
    }
    /// The tip snapshots of these versions, newest first.
    pub fn tips(&self, versions: (u64, u64)) -> Result<Vec<Snapshot>> {
        self.healthy()?;
        let mut tips = Vec::with_capacity(self.tips.len());
        for file in &self.tips {
            let bytes = usize::try_from(file.metadata()?.len())
                .map_err(|_| invalid("snapshot length overflow"))?;
            tips.extend(Snapshot::open(file, 0, bytes, versions)?);
        }
        tips.sort_unstable_by_key(|tip| std::cmp::Reverse(tip.len));
        Ok(tips)
    }
    /// The retained snapshots past `after` through `end`, newest first, chosen by indexed height
    /// before any is opened.
    pub fn between(
        &self,
        after: usize,
        end: usize,
        versions: (u64, u64),
    ) -> impl Iterator<Item = Result<Option<Snapshot>>> + '_ {
        let to = self.entries.partition_point(|&(h, _, _)| h <= end);
        let from = self
            .entries
            .partition_point(|&(h, _, _)| h <= after)
            .min(to);
        self.entries[from..to]
            .iter()
            .rev()
            .map(move |&e| self.read(e, versions))
    }
    pub fn begin(&mut self, start: usize, include_start: bool) -> Result<()> {
        self.healthy()?;
        if !self.writable {
            return Err(invalid("read-only snapshots"));
        }
        self.writing = true;
        let after = |h: usize| h > start || !include_start && h == start;
        let count = self.entries.partition_point(|&(h, _, _)| !after(h));
        if count < self.entries.len() {
            self.entries.truncate(count);
            self.end = self.entries.last().map_or(0, |&(_, at, n)| at + n);
            // Readers open the index before the data: shorten it before the data.
            atomic_write(&self.path.join("pages"), &self.index())?;
            self.data.set_len(self.end)?;
        }
        // A tip past the rewind describes a replaced chain.
        for name in [TIP, PREV] {
            let path = self.path.join(name);
            match File::open(&path) {
                Ok(file) if after(snapshot::stored_len(&file)?) => fs::remove_file(path)?,
                Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
                _ => {}
            }
        }
        self.tips = self.open_tips()?;
        Ok(())
    }
    pub fn push(&mut self, state: &State, versions: (u64, u64), retained: bool) -> Result<()> {
        if !self.writing || !self.writable {
            return Err(invalid("snapshot write outside update"));
        }
        if self.tip_pushed
            || state.len() > 100_000_000
            || self
                .entries
                .last()
                .is_some_and(|&(h, _, _)| h >= state.len())
        {
            return Err(invalid("invalid snapshot append"));
        }
        if retained {
            let n = snapshot::write(&self.data, self.end, state, versions, true)?;
            self.entries.push((state.len(), self.end, n));
            self.end += n;
            self.pages_changed = true;
        } else {
            let file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(self.path.join(NEXT))?;
            snapshot::write(&file, 0, state, versions, false)?;
            self.tip_pushed = true;
        }
        Ok(())
    }
    pub fn finish(&mut self) -> Result<()> {
        if !self.writing || !self.writable {
            return Err(invalid("snapshot commit outside update"));
        }
        if self.pages_changed {
            atomic_write(&self.path.join("pages"), &self.index())?;
            self.pages_changed = false;
        }
        if self.tip_pushed {
            let tip = self.path.join(TIP);
            match fs::rename(&tip, self.path.join(PREV)) {
                Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
                _ => {}
            }
            fs::rename(self.path.join(NEXT), tip)?;
            self.tip_pushed = false;
        }
        self.tips = self.open_tips()?;
        self.writing = false;
        Ok(())
    }
}
fn remove_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}
