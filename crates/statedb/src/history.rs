use crate::{
    Amount, Creations, Cursor, Reader, Spends, State, snapshot,
    snapshot::Snapshot,
    util::{atomic_link, invalid},
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Error, Result},
    path::{Path, PathBuf},
};

/// Owns full snapshots. Spends and creations remain their sole, separately produced diff columns.
pub struct History {
    path: PathBuf,
    _lock: Option<File>,
    interval: usize,
    live: Option<(State, (u64, u64))>,
}
impl History {
    pub const DEFAULT_INTERVAL: usize = 5000;

    pub fn open(path: &Path) -> Result<Self> {
        let path = path.join("snapshots");
        fs::create_dir_all(&path)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.join("writer"))?;
        lock.try_lock().map_err(Error::other)?;
        Ok(Self {
            path,
            _lock: Some(lock),
            interval: Self::DEFAULT_INTERVAL,
            live: None,
        })
    }
    pub(crate) fn open_reader(path: &Path) -> Self {
        Self {
            path: path.join("snapshots"),
            _lock: None,
            interval: Self::DEFAULT_INTERVAL,
            live: None,
        }
    }
    pub fn set_snapshot_interval(&mut self, interval: usize) -> Result<()> {
        if interval == 0 {
            return Err(invalid("zero snapshot interval"));
        }
        self.interval = interval;
        Ok(())
    }
    pub fn seed(&mut self, state: &State, spends: &Spends, created: &Creations) -> Result<()> {
        if state.len() != spends.start()
            || state.len() != created.start()
            || self.path.join("latest").exists()
        {
            return Err(invalid("invalid origin history seed"));
        }
        let versions = (spends.version(), created.version());
        snapshot::write(&self.path.join(state.len().to_string()), state, versions)?;
        atomic_link(
            &self.path.join(state.len().to_string()),
            &self.path.join("latest"),
        )
    }
    pub fn reader<'a>(
        &'a self,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Reader<'a>> {
        let latest = Snapshot::open(
            &self.path.join("latest"),
            (spends.version(), creations.version()),
        )?;
        let (end, hash) = latest
            .as_ref()
            .map_or((0, [0; 32]), |snapshot| (snapshot.len, snapshot.hash()));
        if end > spends.len().min(creations.len()) {
            return Err(invalid("published history exceeds producer prefix"));
        }
        if end > spends.start().max(creations.start())
            && (spends.hash(end - 1)? != hash || creations.read(end - 1)?.0 != hash)
        {
            return Err(invalid("published history does not match producer chain"));
        }
        Ok(Reader {
            history: self,
            spends,
            creations,
            end,
            latest,
        })
    }
    pub(crate) fn restore(
        &self,
        end: usize,
        spends: &Spends,
        created: &Creations,
        latest: Option<&Snapshot>,
    ) -> Result<State> {
        if end > spends.len().min(created.len()) {
            return Err(invalid("incomplete origin contributions"));
        }
        let versions = (spends.version(), created.version());
        let mut choices = Vec::new();
        if let Some(snapshot) = latest.filter(|snapshot| snapshot.len <= end) {
            choices.push((snapshot.len, None));
        }
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            if let Some(len) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<usize>().ok())
                && len <= end
            {
                choices.push((len, Some(entry.path())));
            }
        }
        choices.sort_by(|a, b| b.0.cmp(&a.0));
        let mut state = None;
        for (_, path) in choices {
            let loaded;
            let snapshot = if let Some(path) = path {
                loaded = Snapshot::open(&path, versions)?;
                loaded.as_ref()
            } else {
                latest
            };
            let Some(snapshot) = snapshot.filter(|snapshot| snapshot.len <= end) else {
                continue;
            };
            if snapshot.len > spends.start().max(created.start()) {
                let h = snapshot.len - 1;
                if spends.hash(h)? != snapshot.hash() || created.read(h)?.0 != snapshot.hash() {
                    continue;
                }
            }
            state = Some(snapshot.state()?);
            break;
        }
        let mut state = state.unwrap_or_default();
        if state.len() < spends.start().max(created.start()) {
            return Err(invalid("no snapshot covers the available diff prefix"));
        }
        Self::replay(&mut state, end, spends, created, |_, _| Ok(()))?;
        Ok(state)
    }
    pub fn advance(
        &mut self,
        start: usize,
        end: usize,
        spends: &Spends,
        created: &Creations,
        mut visit: impl FnMut(usize, Amount) -> Result<()>,
    ) -> Result<()> {
        // Taking first ensures failed updates cannot leave advanced reusable state.
        let live = self.live.take();
        if start > end || end > spends.len().min(created.len()) {
            return Err(invalid("invalid origin range"));
        }
        let versions = (spends.version(), created.version());
        let latest = Snapshot::open(&self.path.join("latest"), versions)?;
        let live = live.filter(|(state, stored_versions)| {
            *stored_versions == versions
                && state.len() == start
                && latest
                    .as_ref()
                    .is_some_and(|s| s.len == start && s.hash() == state.hash())
        });
        let mut state = if let Some((mut state, _)) = live {
            // Producer identities may change at the same length after a reorg.
            if Cursor::new(&mut state, end, spends, created).is_ok() {
                state
            } else {
                self.restore(start, spends, created, latest.as_ref())?
            }
        } else {
            self.restore(start, spends, created, latest.as_ref())?
        };
        if start == end
            && latest
                .as_ref()
                .is_some_and(|s| s.len == end && s.hash() == state.hash())
        {
            self.live = Some((state, versions));
            return Ok(());
        }
        // Publish the rewind before replacing descendants. A failed run exposes only this prefix.
        if latest
            .as_ref()
            .is_none_or(|snapshot| snapshot.len != start || snapshot.hash() != state.hash())
        {
            snapshot::write(&self.path.join("latest"), &state, versions)?;
        }
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            if entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|h| h > start)
            {
                fs::remove_file(entry.path())?;
            }
        }
        let mut from = start;
        while from < end {
            let to = (((from / self.interval) + 1) * self.interval).min(end);
            Self::replay(&mut state, to, spends, created, &mut visit)?;
            if to.is_multiple_of(self.interval) {
                snapshot::write(&self.path.join(to.to_string()), &state, versions)?;
            }
            from = to;
        }
        if end > start && end.is_multiple_of(self.interval) {
            atomic_link(&self.path.join(end.to_string()), &self.path.join("latest"))?;
        } else {
            snapshot::write(&self.path.join("latest"), &state, versions)?;
        }
        self.live = Some((state, versions));
        Ok(())
    }
    pub(crate) fn replay(
        state: &mut State,
        end: usize,
        spends: &Spends,
        created: &Creations,
        mut visit: impl FnMut(usize, Amount) -> Result<()>,
    ) -> Result<()> {
        let mut cursor = Cursor::new(state, end, spends, created)?;
        while cursor.advance()?.is_some() {
            let state = cursor.state();
            visit(state.len() - 1, state.total())?;
        }
        Ok(())
    }
}
