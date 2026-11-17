use crate::{
    Amount, Creations, Cursor, Diffs, Reader, Spends, State,
    snapshot::{PAGE, Snapshot},
    snapshots::Snapshots,
    util::invalid,
};
use std::{
    fs,
    io::Result,
    path::Path,
    sync::{Arc, Mutex},
};

/// Blocks between full snapshots: a restore loads the nearest one and replays fewer than this many
/// diffs. A history laid out at another interval lays its snapshots out again on its next advance.
pub const SNAPSHOT_INTERVAL: usize = 500;

/// The producer's state after its last advance and the versions it was made from, shared with
/// the views of its history: a read near the tip copies it instead of reading the tip snapshot.
/// The producer takes it while it advances, so it never copies the state for a block.
#[derive(Clone, Default)]
pub struct Live(Arc<Mutex<Option<Versioned>>>);
/// A state and the producers' versions it was made from.
type Versioned = (Arc<State>, (u64, u64));
impl Live {
    fn take(&self) -> Option<Versioned> {
        self.0.lock().unwrap().take()
    }
    fn get(&self) -> Option<Versioned> {
        self.0.lock().unwrap().clone()
    }
    fn set(&self, state: State, versions: (u64, u64)) {
        *self.0.lock().unwrap() = Some((Arc::new(state), versions));
    }
}

/// Owns full snapshots. Spends and creations remain their sole, separately produced diff columns.
pub struct History {
    snapshots: Snapshots,
    live: Live,
}
impl History {
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        Ok(Self {
            snapshots: Snapshots::open(&path.join("snapshots"), true)?,
            live: Live::default(),
        })
    }
    pub(crate) fn open_reader(path: &Path, live: Live) -> Result<Self> {
        Ok(Self {
            snapshots: Snapshots::open(&path.join("snapshots"), false)?,
            live,
        })
    }
    /// The producer's state, for the views of this history.
    pub fn live(&self) -> Live {
        self.live.clone()
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
    /// can leave a newer one behind. The live state stands in for those it is as new as: copying
    /// it and taking a block or two back beats reading one from disk.
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
        if let Some((state, versions)) = self.live.get()
            && versions == (spends.version(), creations.version())
            && state.len() <= limit
            && continues(state.len(), state.hash(), spends, creations)?
        {
            tips.retain(|tip| tip.len > state.len());
            tips.push(Snapshot::live(state, versions));
        }
        Ok(tips)
    }
    fn matches(snapshot: &Snapshot, spends: &Spends, creations: &Creations) -> Result<bool> {
        continues(snapshot.len, snapshot.hash(), spends, creations)
    }
    /// The state of the first `end` blocks, from the nearest snapshot within `limit` blocks: one
    /// at or below `end` replays the blocks up to it, one above takes the blocks past it back.
    /// `tips` are the validated tips of `tips_within`, newest first.
    pub(crate) fn restore(
        &self,
        end: usize,
        limit: usize,
        spends: &Spends,
        created: &Creations,
        tips: &[Snapshot],
    ) -> Result<State> {
        debug_assert!(end <= limit && limit <= spends.end().min(created.end()));
        let versions = (spends.version(), created.version());
        let tip = tips.iter().find(|tip| tip.len <= end);
        let floor = tip.map_or(0, |tip| tip.len);
        let retained =
            Self::first_matching(self.snapshots.between(floor, end, versions), spends, created)?;
        let below = retained.as_ref().or(tip);
        let replay = end - below.map_or(0, |snapshot| snapshot.len);
        // One above serves only when strictly nearer.
        let bound = limit.min(end + replay.saturating_sub(1));
        if let Some(mut state) = self.state_above(end, bound, spends, created, tips)? {
            Self::undo(&mut state, end, spends, created)?;
            return Ok(state);
        }
        let mut state = below.map_or(Ok(State::default()), Snapshot::state)?;
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
        // Snapshots missing below `start` (another interval's layout, or a relayout cut short) are
        // laid out again by a replay from the last one in place that visits only `start..end`. It
        // rewrites snapshots below open views' ceilings, so it runs before any reader opens a view:
        // the first advance after the interval changes is bitview's bootstrap, before it serves.
        let from = self.snapshots.laid_out_through(SNAPSHOT_INTERVAL, start);
        let warm = live.filter(|(state, old)| {
            *old == versions
                && state.len() == from
                && continues(from, state.hash(), spends, created).is_ok_and(|ok| ok)
        });
        let (mut state, warm) = match warm {
            Some((state, _)) => (Arc::unwrap_or_clone(state), true),
            None => {
                let tips = self.tips_within(spends, created, from)?;
                (self.restore(from, from, spends, created, &tips)?, false)
            }
        };
        if from == end
            && (warm
                || self
                    .snapshots
                    .latest(versions)?
                    .is_some_and(|s| s.len == end && s.hash() == state.hash()))
        {
            self.live.set(state, versions);
            return Ok(());
        }
        let base = (from, state.hash());
        self.snapshots.truncate(from)?;
        // Pages of the tip that differ from the state at `from`: grown ones and spent ones.
        let mut changed = vec![false; end.div_ceil(PAGE)];
        changed[from / PAGE..].fill(true);
        let mut visit = |height, amount| {
            if height < start { Ok(()) } else { visit(height, amount) }
        };
        let mut height = from;
        loop {
            if height > 0 && height.is_multiple_of(SNAPSHOT_INTERVAL) {
                self.snapshots.retain(&state, versions)?;
            }
            if height == end {
                break;
            }
            height = ((height / SNAPSHOT_INTERVAL + 1) * SNAPSHOT_INTERVAL).min(end);
            Self::replay(
                &mut state,
                height,
                spends,
                created,
                |origin| changed[origin as usize / PAGE] = true,
                &mut visit,
            )?;
        }
        self.snapshots.publish(&state, versions, base, &changed)?;
        self.live.set(state, versions);
        Ok(())
    }
    /// Drops the snapshots past `len`, as a history laid out at another interval lacks some.
    #[cfg(test)]
    pub(crate) fn drop_snapshots_past(&mut self, len: usize) -> Result<()> {
        self.snapshots.truncate(len)
    }
    #[cfg(test)]
    pub(crate) fn snapshot_heights(&self) -> Vec<usize> {
        self.snapshots.heights()
    }
    /// The state of the nearest snapshot past `end` through `bound`, tip or retained.
    fn state_above(
        &self,
        end: usize,
        bound: usize,
        spends: &Spends,
        created: &Creations,
        tips: &[Snapshot],
    ) -> Result<Option<State>> {
        if bound <= end {
            return Ok(None);
        }
        let tip = tips.iter().rev().find(|tip| tip.len > end && tip.len <= bound);
        let upto = tip.map_or(bound, |tip| tip.len);
        // `between` runs newest first.
        let above = self.snapshots.between(end, upto, (spends.version(), created.version()));
        let retained = Self::first_matching(above.rev(), spends, created)?;
        retained.as_ref().or(tip).map(Snapshot::state).transpose()
    }
    /// The first of `snapshots` on the producers' chain.
    fn first_matching(
        snapshots: impl Iterator<Item = Result<Option<Snapshot>>>,
        spends: &Spends,
        created: &Creations,
    ) -> Result<Option<Snapshot>> {
        for snapshot in snapshots {
            if let Some(snapshot) = snapshot?
                && Self::matches(&snapshot, spends, created)?
            {
                return Ok(Some(snapshot));
            }
        }
        Ok(None)
    }
    /// Takes `state`, a snapshot of the producer chain, back to its first `end` blocks: the later
    /// blocks' origins leave, and what they removed from the earlier ones returns.
    fn undo(state: &mut State, end: usize, spends: &Spends, created: &Creations) -> Result<()> {
        let (len, hash) = (state.len(), state.hash());
        let parent = spends.hash(end - 1)?;
        if created.hash(end - 1)? != parent {
            return Err(invalid("origin producer chain mismatch"));
        }
        state.truncate(end, parent);
        let mut diffs = Diffs::new(end, Some(parent), len, spends, created)?;
        while let Some(diff) = diffs.advance()? {
            state.unremove(&diff);
        }
        if diffs.hash() != Some(hash) {
            return Err(invalid("snapshot does not continue the producer chain"));
        }
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
