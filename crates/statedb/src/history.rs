use crate::{
    Amount, Creations, Cursor, Reader, Spends, State, snapshot::Snapshot, snapshots::Snapshots,
    util::invalid,
};
use std::{fs, io::Result, path::Path};

/// Blocks between full snapshots: a restore loads the nearest one and replays at most this many diffs.
pub const SNAPSHOT_INTERVAL: usize = 5000;

/// Owns full snapshots. Spends and creations remain their sole, separately produced diff columns.
pub struct History {
    snapshots: Snapshots,
    interval: usize,
    live: Option<(State, (u64, u64))>,
}
impl History {
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        Ok(Self {
            snapshots: Snapshots::open(&path.join("snapshots"), true)?,
            interval: SNAPSHOT_INTERVAL,
            live: None,
        })
    }
    pub(crate) fn open_reader(path: &Path) -> Result<Self> {
        Ok(Self {
            snapshots: Snapshots::open(&path.join("snapshots"), false)?,
            interval: SNAPSHOT_INTERVAL,
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
    /// A reader of the published history: as far as its newest snapshot the producers carry.
    pub fn reader<'a>(
        &'a self,
        spends: &'a Spends,
        creations: &'a Creations,
    ) -> Result<Reader<'a>> {
        let limit = spends.end().min(creations.end());
        let tips = self.tips_within(spends, creations, limit)?;
        let mut end = tips.first().map_or(0, |tip| tip.len);
        let versions = (spends.version(), creations.version());
        for snapshot in self.snapshots.before(limit, versions) {
            if let Some(snapshot) = snapshot?
                && Self::matches(&snapshot, spends, creations)?
            {
                end = end.max(snapshot.len);
                break;
            }
        }
        Ok(Reader {
            history: self,
            spends,
            creations,
            end,
            tips,
        })
    }
    /// A reader through `ceiling` blocks (or the producers' end), reading nothing past them: a
    /// rewind may be rewriting the producers' later rows. Snapshots only speed up its restores.
    pub(crate) fn reader_within<'a>(
        &'a self,
        spends: &'a Spends,
        creations: &'a Creations,
        ceiling: usize,
    ) -> Result<Reader<'a>> {
        let end = spends.end().min(creations.end()).min(ceiling);
        let tips = self.tips_within(spends, creations, end)?;
        Ok(Reader {
            history: self,
            spends,
            creations,
            end,
            tips,
        })
    }
    /// The tip snapshots the producers carry within `limit`, newest first: a rewind in progress
    /// can leave a newer one behind.
    fn tips_within(
        &self,
        spends: &Spends,
        creations: &Creations,
        limit: usize,
    ) -> Result<Vec<Snapshot>> {
        let mut tips = Vec::new();
        for tip in self
            .snapshots
            .tips((spends.version(), creations.version()))?
        {
            if tip.len <= limit && Self::matches(&tip, spends, creations)? {
                tips.push(tip);
            }
        }
        Ok(tips)
    }
    fn matches(snapshot: &Snapshot, spends: &Spends, created: &Creations) -> Result<bool> {
        if snapshot.len == 0 {
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
        tips: &[Snapshot],
    ) -> Result<State> {
        if end > spends.end().min(created.end()) {
            return Err(invalid("incomplete origin contributions"));
        }
        // The newest matching snapshot, tip or retained, leaves the least to replay.
        let mut newest = None;
        for tip in tips.iter().filter(|tip| tip.len <= end) {
            if Self::matches(tip, spends, created)? {
                newest = Some(tip);
                break;
            }
        }
        let mut retained = None;
        for snapshot in self
            .snapshots
            .before(end, (spends.version(), created.version()))
        {
            let Some(snapshot) = snapshot? else {
                continue;
            };
            if newest.is_some_and(|tip| tip.len >= snapshot.len) {
                break;
            }
            if Self::matches(&snapshot, spends, created)? {
                retained = Some(snapshot);
                break;
            }
        }
        let mut state = match (retained, newest) {
            (Some(snapshot), _) => snapshot.state()?,
            (None, Some(tip)) => tip.state()?,
            (None, None) => State::default(),
        };
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
            self.restore(start, spends, created, &self.snapshots.tips(versions)?)?
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
            self.snapshots
                .push(&state, versions, end.is_multiple_of(self.interval))?;
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
