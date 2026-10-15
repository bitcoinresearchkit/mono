use crate::{
    Amount, Creations, Cursor, Reader, Spends, State,
    snapshot::{PAGE, Snapshot},
    snapshots::Snapshots,
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
        for snapshot in self.snapshots.between(end, limit, versions) {
            if let Some(snapshot) = snapshot?
                && Self::matches(&snapshot, spends, creations)?
            {
                end = snapshot.len;
                break;
            }
        }
        Ok(Reader {
            history: self,
            spends,
            creations,
            end,
            tips,
            checked: Default::default(),
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
            checked: Default::default(),
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
    fn matches(snapshot: &Snapshot, spends: &Spends, creations: &Creations) -> Result<bool> {
        continues(snapshot.len, snapshot.hash(), spends, creations)
    }
    /// `tips` are the validated tips of `tips_within`, newest first.
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
        let newest = tips.iter().find(|tip| tip.len <= end);
        let mut retained = None;
        for snapshot in self.snapshots.between(
            newest.map_or(0, |tip| tip.len),
            end,
            (spends.version(), created.version()),
        ) {
            if let Some(snapshot) = snapshot?
                && Self::matches(&snapshot, spends, created)?
            {
                retained = Some(snapshot);
                break;
            }
        }
        let mut state = retained
            .as_ref()
            .or(newest)
            .map_or(Ok(State::default()), Snapshot::state)?;
        if state.len() < end {
            Self::replay(&mut state, end, spends, created, |_| {}, |_, _| Ok(()))?;
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
        let warm = live.filter(|(state, old)| {
            *old == versions
                && state.len() == start
                && continues(start, state.hash(), spends, created).is_ok_and(|ok| ok)
        });
        let (mut state, warm) = match warm {
            Some((state, _)) => (state, true),
            None => {
                let tips = self.tips_within(spends, created, start)?;
                (self.restore(start, spends, created, &tips)?, false)
            }
        };
        if start == end
            && (warm
                || self
                    .snapshots
                    .latest(versions)?
                    .is_some_and(|s| s.len == end && s.hash() == state.hash()))
        {
            self.live = Some((state, versions));
            return Ok(());
        }
        let from = (start, state.hash());
        self.snapshots.truncate(start)?;
        // Pages of the tip that differ from the state at `start`: grown ones and spent ones.
        let mut changed = vec![false; end.div_ceil(PAGE)];
        changed[start / PAGE..].fill(true);
        let mut height = start;
        loop {
            if height > 0 && height.is_multiple_of(self.interval) {
                self.snapshots.retain(&state, versions)?;
            }
            if height == end {
                break;
            }
            height = ((height / self.interval + 1) * self.interval).min(end);
            Self::replay(
                &mut state,
                height,
                spends,
                created,
                |origin| changed[origin as usize / PAGE] = true,
                &mut visit,
            )?;
        }
        self.snapshots.publish(&state, versions, from, &changed)?;
        self.live = Some((state, versions));
        Ok(())
    }
    /// Replays `state`, which continues the producer chain, through `end`.
    fn replay(
        state: &mut State,
        end: usize,
        spends: &Spends,
        created: &Creations,
        mut removed: impl FnMut(u32),
        mut visit: impl FnMut(usize, Amount) -> Result<()>,
    ) -> Result<()> {
        let mut cursor = Cursor::new(state, end, spends, created)?;
        while let Some(diff) = cursor.advance()? {
            diff.removed().for_each(|(origin, _)| removed(origin));
            let state = cursor.state();
            visit(state.len() - 1, state.total())?;
        }
        Ok(())
    }
}
/// Whether the state of `len` blocks ending in `hash` is a prefix of both producers' chain.
pub(crate) fn continues(
    len: usize,
    hash: [u8; 32],
    spends: &Spends,
    creations: &Creations,
) -> Result<bool> {
    Ok(len == 0 || spends.hash(len - 1)? == hash && creations.hash(len - 1)? == hash)
}
