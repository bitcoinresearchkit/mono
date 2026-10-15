use crate::{
    State,
    snapshot::{self, HEADER, MAX_RECORD, PAGE, PAGE_BYTES, PAGED, Snapshot},
    util::{atomic_write, invalid},
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Error, ErrorKind, Result, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
};
const MAGIC: &[u8; 8] = b"STAPAGE4";
const INDEX_HEADER: usize = 9;
const ROW: usize = 12;
const TIP: &str = "tip";
const PREV: &str = "prev";
const NEXT: &str = "tip.next";
/// Tip page files are `tips.<generation>`.
const PAGES: &str = "tips.";
/// A tip's page file grows to this many times its live pages before the next tip starts a fresh one.
const GARBAGE: u64 = 4;

/// A published tip: its directory and the page file it points into, opened together.
struct Tip {
    header: [u8; HEADER],
    dir: File,
    pages: File,
    len: usize,
}
impl Tip {
    fn open(path: &Path, name: &str, writable: bool) -> Result<Option<Self>> {
        let dir = match File::open(path.join(name)) {
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            dir => dir?,
        };
        let mut header = [0; HEADER];
        dir.read_exact_at(&mut header, 0)?;
        // Tips of older formats are rebuilt: restores use retained snapshots meanwhile. A writer
        // removes page files no tip refers to: a reader racing it reads without that tip.
        let pages = if &header[..8] == PAGED {
            let pages = OpenOptions::new()
                .read(true)
                .write(writable)
                .open(pages_path(path, snapshot::generation(&header)));
            match pages {
                Err(error) if error.kind() == ErrorKind::NotFound => None,
                pages => Some(pages?),
            }
        } else {
            None
        };
        let Some(pages) = pages else {
            if writable {
                fs::remove_file(path.join(name))?;
            }
            return Ok(None);
        };
        let len = snapshot::len(&header)?;
        Ok(Some(Self {
            header,
            dir,
            pages,
            len,
        }))
    }
    fn generation(&self) -> u64 {
        snapshot::generation(&self.header)
    }
    fn hash(&self) -> [u8; 32] {
        snapshot::hash(&self.header)
    }
    fn versions(&self) -> (u64, u64) {
        snapshot::versions(&self.header)
    }
}
fn pages_path(path: &Path, generation: u64) -> PathBuf {
    path.join(format!("{PAGES}{generation}"))
}

/// Full snapshots, published so readers never see one change. Retained snapshots are appended to
/// `data` and listed by `pages` (replaced by rename); only a rewind truncates them. The newest
/// snapshot is the `tip` and the one it replaced is `prev`, so a reader still at the previous
/// publication restores from a snapshot too. A tip is a page directory (replaced by rename) into
/// a page file of immutable slots: a tip appends only the pages it changed, and starts a fresh
/// page file once the old one is mostly garbage. Readers keep the files they opened.
pub(crate) struct Snapshots {
    path: PathBuf,
    data: File,
    writable: bool,
    entries: Vec<(usize, u64, u64)>,
    end: u64,
    tip: Option<Tip>,
    prev: Option<Tip>,
    /// The writer's next free slot in the tip's page file.
    slots: u64,
    /// The newest page file generation.
    generation: u64,
    poisoned: bool,
}
impl Snapshots {
    pub fn open(path: &Path, writable: bool) -> Result<Self> {
        if writable {
            fs::create_dir_all(path)?;
        }
        let data = OpenOptions::new()
            .create(writable)
            .truncate(false)
            .read(true)
            .write(writable)
            .open(path.join("data"))?;
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
        if bytes.len() < INDEX_HEADER
            || bytes.len() as u64 > MAX_RECORD
            || !(bytes.len() - INDEX_HEADER).is_multiple_of(ROW)
        {
            return Err(invalid("invalid snapshot page index length"));
        }
        if &bytes[..8] != MAGIC {
            return Err(invalid("unsupported snapshot page index"));
        }
        let retained_last = bytes[8];
        if retained_last > 1 || bytes.len() == INDEX_HEADER && retained_last != 0 {
            return Err(invalid("invalid snapshot retention"));
        }
        let mut entries: Vec<(usize, u64, u64)> =
            Vec::with_capacity((bytes.len() - INDEX_HEADER) / ROW);
        let mut end = 0;
        for row in bytes[INDEX_HEADER..].as_chunks::<ROW>().0 {
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
        let tip = Tip::open(path, TIP, writable)?;
        let prev = Tip::open(path, PREV, writable)?;
        let this = Self {
            path: path.to_owned(),
            data,
            writable,
            entries,
            end,
            slots: match &tip {
                Some(tip) => tip.pages.metadata()?.len().div_ceil(PAGE_BYTES),
                None => 0,
            },
            generation: tip
                .iter()
                .chain(&prev)
                .map(Tip::generation)
                .max()
                .unwrap_or(0),
            tip,
            prev,
            poisoned: false,
        };
        if writable {
            if len != end {
                this.data.set_len(end)?;
                atomic_write(&pages_path, &this.index())?;
            }
            remove_if_exists(&path.join(NEXT))?;
            this.remove_unused_pages()?;
        }
        Ok(this)
    }
    fn index(&self) -> Vec<u8> {
        let mut index = Vec::with_capacity(INDEX_HEADER + self.entries.len() * ROW);
        index.extend_from_slice(MAGIC);
        index.push(u8::from(!self.entries.is_empty()));
        for &(h, at, n) in &self.entries {
            index.extend_from_slice(&(h as u32).to_le_bytes());
            index.extend_from_slice(&(at + n).to_le_bytes());
        }
        index
    }
    fn healthy(&self) -> Result<()> {
        if self.poisoned {
            Err(invalid(
                "snapshot update did not finish; reopen before reuse",
            ))
        } else {
            Ok(())
        }
    }
    /// Runs one write; a failure leaves the files mid-update, so later use must reopen.
    fn write(&mut self, f: impl FnOnce(&mut Self) -> Result<()>) -> Result<()> {
        self.healthy()?;
        if !self.writable {
            return Err(invalid("read-only snapshots"));
        }
        let result = f(self);
        self.poisoned = result.is_err();
        result
    }
    fn read(&self, entry: (usize, u64, u64), versions: (u64, u64)) -> Result<Option<Snapshot>> {
        self.healthy()?;
        let (h, at, n) = entry;
        let snapshot = Snapshot::open_packed(&self.data, at, n as usize, versions)?;
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
        let mut tips = Vec::with_capacity(2);
        for tip in self.tip.iter().chain(&self.prev) {
            tips.extend(Snapshot::open_paged(
                tip.header, &tip.dir, &tip.pages, versions,
            )?);
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
    /// Drops every snapshot past `start`: after a rewind they describe a replaced chain.
    pub fn truncate(&mut self, start: usize) -> Result<()> {
        self.write(|this| {
            let count = this.entries.partition_point(|&(h, _, _)| h <= start);
            if count < this.entries.len() {
                this.entries.truncate(count);
                this.end = this.entries.last().map_or(0, |&(_, at, n)| at + n);
                // Readers open the index before the data: shorten it before the data.
                atomic_write(&this.path.join("pages"), &this.index())?;
                this.data.set_len(this.end)?;
            }
            for (name, tip) in [(TIP, &mut this.tip), (PREV, &mut this.prev)] {
                if tip.as_ref().is_some_and(|tip| tip.len > start) {
                    fs::remove_file(this.path.join(name))?;
                    *tip = None;
                }
            }
            Ok(())
        })
    }
    /// Appends `state` as a retained snapshot, unless one at its height already is.
    pub fn retain(&mut self, state: &State, versions: (u64, u64)) -> Result<()> {
        self.write(|this| {
            match this.entries.last() {
                Some(&(h, _, _)) if h == state.len() => return Ok(()),
                last => debug_assert!(last.is_none_or(|&(h, _, _)| h < state.len())),
            }
            let n = snapshot::write_packed(&this.data, this.end, state, versions)?;
            this.entries.push((state.len(), this.end, n));
            this.end += n;
            atomic_write(&this.path.join("pages"), &this.index())
        })
    }
    /// Publishes `state` as the tip. `changed` flags the pages that differ from the state at
    /// `from` (its length and hash): when that is the tip, only those pages are appended.
    pub fn publish(
        &mut self,
        state: &State,
        versions: (u64, u64),
        from: (usize, [u8; 32]),
        changed: &[bool],
    ) -> Result<()> {
        debug_assert_eq!(changed.len(), state.len().div_ceil(PAGE));
        self.write(|this| {
            let count = changed.len() as u64;
            let appended = changed.iter().filter(|&&c| c).count() as u64;
            let amounts = state.amounts();
            let base = this.tip.as_ref().filter(|tip| {
                (tip.len, tip.hash(), tip.versions()) == (from.0, from.1, versions)
                    && this.slots + appended <= GARBAGE * count
            });
            let (generation, file, slots) = if let Some(tip) = base {
                let mut slots = snapshot::read_slots(&tip.dir, tip.len)?;
                slots.resize(changed.len(), 0);
                // Changed pages land in consecutive fresh slots: one write.
                let mut buffer = Vec::with_capacity(appended as usize * PAGE);
                let mut next = this.slots;
                for (page, slot) in slots.iter_mut().enumerate().filter(|(p, _)| changed[*p]) {
                    *slot = next as u32;
                    next += 1;
                    buffer.extend_from_slice(
                        &amounts[page * PAGE..amounts.len().min((page + 1) * PAGE)],
                    );
                }
                snapshot::write_amounts(&tip.pages, this.slots * PAGE_BYTES, &buffer)?;
                this.slots = next;
                (tip.generation(), tip.pages.try_clone()?, slots)
            } else {
                this.generation += 1;
                let file = OpenOptions::new()
                    .create_new(true)
                    .read(true)
                    .write(true)
                    .open(pages_path(&this.path, this.generation))?;
                snapshot::write_amounts(&file, 0, amounts)?;
                this.slots = count;
                (this.generation, file, (0..count as u32).collect())
            };
            let fresh = base.is_none();
            let header = snapshot::header(PAGED, state, versions, generation);
            let mut bytes = Vec::with_capacity(HEADER + slots.len() * 4);
            bytes.extend_from_slice(&header);
            slots
                .iter()
                .for_each(|slot| bytes.extend_from_slice(&slot.to_le_bytes()));
            let next = this.path.join(NEXT);
            let mut dir = OpenOptions::new()
                .create(true)
                .truncate(true)
                .read(true)
                .write(true)
                .open(&next)?;
            dir.write_all(&bytes)?;
            if this.tip.is_some() {
                fs::rename(this.path.join(TIP), this.path.join(PREV))?;
                this.prev = this.tip.take();
            }
            fs::rename(next, this.path.join(TIP))?;
            this.tip = Some(Tip {
                header,
                dir,
                pages: file,
                len: state.len(),
            });
            if fresh {
                this.remove_unused_pages()?;
            }
            Ok(())
        })
    }
    /// Removes the page files neither the tip nor `prev` points into.
    fn remove_unused_pages(&self) -> Result<()> {
        let used = [&self.tip, &self.prev].map(|tip| tip.as_ref().map(Tip::generation));
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let generation = entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_prefix(PAGES)?.parse::<u64>().ok());
            if generation.is_some_and(|g| !used.contains(&Some(g))) {
                fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}
fn remove_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}
