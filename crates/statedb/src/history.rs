use crate::{
    Amount, Creations, Cursor, Reader, Spends, State, snapshot::Snapshot, snapshots::Snapshots,
    util::invalid,
};
use std::{fs, io::Result, path::Path};

/// Owns full snapshots. Spends and creations remain their sole, separately produced diff columns.
pub struct History {
    snapshots: Snapshots,
    interval: usize,
    live: Option<(State, (u64, u64))>,
}
impl History {
    const DEFAULT_INTERVAL: usize = 5000;
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        Ok(Self {
            snapshots: Snapshots::open(&path.join("snapshots"), true)?,
            interval: Self::DEFAULT_INTERVAL,
            live: None,
        })
    }
    pub(crate) fn open_reader(path: &Path) -> Result<Self> {
        Ok(Self {
            snapshots: Snapshots::open(&path.join("snapshots"), false)?,
            interval: Self::DEFAULT_INTERVAL,
            live: None,
        })
    }
    #[cfg(test)]
    pub(crate) fn set_snapshot_interval(&mut self, interval: usize) -> Result<()> {
        if interval == 0 {
            return Err(invalid("zero snapshot interval"));
        }
        self.interval = interval;
        Ok(())
    }
    pub fn reader<'a>(
        &'a self,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Reader<'a>> {
        let latest = self
            .snapshots
            .latest((spends.version(), creations.version()))?;
        let (end, hash) = latest.as_ref().map_or((0, [0; 32]), |s| (s.len, s.hash()));
        if end > spends.end().min(creations.end()) {
            return Err(invalid("published history exceeds producer prefix"));
        }
        if end > spends.start().max(creations.start())
            && (spends.hash(end - 1)? != hash || creations.hash(end - 1)? != hash)
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
    fn matches(snapshot: &Snapshot, spends: &Spends, created: &Creations) -> Result<bool> {
        if snapshot.len <= spends.start().max(created.start()) {
            return Ok(true);
        }
        let h = snapshot.len - 1;
        Ok(spends.hash(h)? == snapshot.hash() && created.hash(h)? == snapshot.hash())
    }
    pub(crate) fn restore(
        &self,
        end: usize,
        spends: &Spends,
        created: &Creations,
        latest: Option<&Snapshot>,
    ) -> Result<State> {
        if end > spends.end().min(created.end()) {
            return Err(invalid("incomplete origin contributions"));
        }
        let mut state = None;
        if let Some(snapshot) = latest.filter(|s| s.len <= end)
            && Self::matches(snapshot, spends, created)?
        {
            state = Some(snapshot.state()?);
        }
        if state.is_none() {
            for snapshot in self
                .snapshots
                .before(end, (spends.version(), created.version()))
            {
                let Some(snapshot) = snapshot? else {
                    continue;
                };
                if Self::matches(&snapshot, spends, created)? {
                    state = Some(snapshot.state()?);
                    break;
                }
            }
        }
        let mut state = state.unwrap_or_default();
        if state.len() < spends.start().max(created.start()) {
            return Err(invalid("no snapshot covers the available diff prefix"));
        }
        if state.len() < end {
            Self::replay(&mut state, end, spends, created, |_, _| Ok(()))?;
        }
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
        let live = self.live.take();
        if start > end || end > spends.end().min(created.end()) {
            return Err(invalid("invalid origin range"));
        }
        let versions = (spends.version(), created.version());
        let cached = live.filter(|(state, old)| *old == versions && state.len() == start);
        let cached = cached.and_then(|(mut state, _)| {
            Cursor::new(&mut state, end, spends, created)
                .is_ok()
                .then_some(state)
        });
        let warm = cached.is_some();
        let latest = if warm {
            None
        } else {
            self.snapshots.latest(versions)?
        };
        let mut state = if let Some(state) = cached {
            state
        } else {
            self.restore(start, spends, created, latest.as_ref())?
        };
        if start == end
            && (warm
                || latest
                    .as_ref()
                    .is_some_and(|s| s.len == end && s.hash() == state.hash()))
        {
            self.live = Some((state, versions));
            return Ok(());
        }
        let mut from = start;
        let mut saved = start;
        while from < end {
            let to = (((from / self.interval) + 1) * self.interval).min(end);
            Self::replay(&mut state, to, spends, created, &mut visit)?;
            if to.is_multiple_of(self.interval) {
                self.snapshots.begin(saved, true)?;
                self.snapshots.push(&state, versions, true)?;
                self.snapshots.finish()?;
                saved = to;
            }
            from = to;
        }
        if start == end || !end.is_multiple_of(self.interval) {
            self.snapshots.begin(saved, start < end)?;
            self.snapshots.push(
                &state,
                versions,
                end.is_multiple_of(self.interval) || end == spends.start().max(created.start()),
            )?;
            self.snapshots.finish()?;
        }
        self.live = Some((state, versions));
        Ok(())
    }
    fn replay(
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
