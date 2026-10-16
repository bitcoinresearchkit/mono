//! The storage stack (vecdb over rawdb) against a plain in-memory model. Random operation
//! sequences run over every variant: raw, compressed, mutable and overflow vectors, with lazy
//! views over the dense ones. Every read API is compared after each step, the vector and the
//! whole database reopen at random, and readers on other threads must see whole values while a
//! writer appends and rewrites.

use std::{
    any::type_name,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
    thread,
};

use crate::{
    AnyStoredVec, AnyVec, Budgeted, BytesVec, CompressionStrategy, Database, DeltaSub, Header,
    ImportOptions, LazyDeltaVec, LazyVec, MutableVec, OverflowVec, OverflowVecValue, RawStrategy,
    ReadWriteCompressedVec, ReadWriteRawVec, ReadableCloneableVec, ReadableVec, Stamp, StoredVec,
    Version, cache::CachePolicy,
};

/// Undo records each vector keeps: deeper rollbacks fail to land and rebuild instead.
const KEPT: u16 = 3;
const STEPS: usize = 2_000;

#[test]
fn storage_matches_model() {
    // Small enough that budgeted sources evict while they are read.
    Budgeted::init_global(256 * 1024).unwrap();
    let tmp = TempDir(std::env::temp_dir().join(format!("vecdb-test-{}", process::id())));
    let dir = tmp.0.as_path();
    thread::scope(|scope| {
        scope.spawn(|| run::<BytesVec<usize, u64>>(dir, 1));
        scope.spawn(|| run::<BytesVec<usize, u64, Budgeted>>(dir, 2));
        scope.spawn(|| run::<MutableVec<BytesVec<usize, u64>>>(dir, 3));
        scope.spawn(|| run::<OverflowVec<usize, u64>>(dir, 4));
        #[cfg(feature = "zerocopy")]
        scope.spawn(|| run::<crate::ZeroCopyVec<usize, u64>>(dir, 5));
        #[cfg(feature = "pco")]
        scope.spawn(|| run::<crate::PcoVec<usize, u64>>(dir, 6));
        #[cfg(feature = "pco")]
        scope.spawn(|| run::<crate::PcoVec<usize, u64, Budgeted>>(dir, 7));
        #[cfg(feature = "zstd")]
        scope.spawn(|| run::<crate::ZstdVec<usize, u64>>(dir, 8));
        #[cfg(feature = "lz4")]
        scope.spawn(|| run::<crate::LZ4Vec<usize, u64>>(dir, 9));
    });
}

/// Two vectors of one variant share a database, so each one's growth relocates the other.
fn run<V: Variant>(dir: &Path, seed: u64) {
    let dir = dir.join(seed.to_string());
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut db = Database::open(&dir).unwrap();
    let mut subjects: Vec<Subject<V>> = [("a", false), ("b", true)]
        .into_iter()
        .map(|(name, unread)| Subject::open(&db, State::new(name, unread)))
        .collect();
    for _ in 0..STEPS {
        let i = rng.below(subjects.len());
        match rng.below(100) {
            // A restart: every vector closes, then the database reopens.
            0 => {
                let states: Vec<_> = subjects
                    .drain(..)
                    .map(|s| s.close(&mut rng, false))
                    .collect();
                drop(db);
                db = Database::open(&dir).unwrap();
                subjects = states.into_iter().map(|s| Subject::open(&db, s)).collect();
                // The imports touched every region the vectors keep.
                db.retain_accessed_regions().unwrap();
                subjects.iter().for_each(|s| s.check(&mut rng));
            }
            // One vector reopens, sometimes under a new version (its data resets).
            1..=4 => {
                let bump = rng.chance(16);
                let state = subjects.swap_remove(i).close(&mut rng, bump);
                subjects.push(Subject::open(&db, state));
                subjects.last().unwrap().check(&mut rng);
            }
            5 => db.compact().unwrap(),
            6 => db.flush(),
            _ => {
                subjects[i].step(&mut rng);
                subjects[i].check(&mut rng);
            }
        }
    }
    concurrent::<V>(&db, &mut rng);
}

/// One vector under test, with a read-only clone kept across its writes.
struct Subject<V: Variant> {
    vec: V,
    clone: V::ReadOnly,
    state: State,
}

/// What survives closing a vector: its import options and its model.
struct State {
    name: &'static str,
    version: u32,
    /// Imports with unread updates (only the rawdb write path differs here).
    unread: bool,
    model: Model,
}

impl State {
    fn new(name: &'static str, unread: bool) -> Self {
        Self {
            name,
            version: 1,
            unread,
            model: Model::default(),
        }
    }
}

impl<V: Variant> Subject<V> {
    fn open(db: &Database, state: State) -> Self {
        let options = ImportOptions::new(db, state.name, Version::new(state.version))
            .with_saved_stamped_changes(KEPT);
        let options = if state.unread {
            // SAFETY: the vector and its clones are read only between writes, on this thread.
            unsafe { options.with_unread_updates() }
        } else {
            options
        };
        let vec = V::import_with(options).unwrap();
        Self {
            clone: vec.read_only_clone(),
            vec,
            state,
        }
    }

    /// Closes the vector as a restart does. A plain write since the last checkpoint is
    /// checkpointed first: undo records cannot follow it across a restart.
    fn close(mut self, rng: &mut Rng, bump: bool) -> State {
        if self.state.model.disk != self.state.model.disk_base {
            checkpoint(&mut self.vec, &mut self.state.model, rng);
        }
        let mut state = self.state;
        state.model.reopen(bump);
        state.version += u32::from(bump);
        state
    }

    fn step(&mut self, rng: &mut Rng) {
        let Self { vec, state, .. } = self;
        let m = &mut state.model;
        let len = m.cur.len();
        match rng.below(100) {
            0..=39 => {
                let count = match rng.below(8) {
                    0 | 1 if len < 60_000 => 2_000 + rng.below(10_000),
                    2 | 3 => rng.below(200),
                    _ => 1 + rng.below(8),
                };
                let values = values(rng, count);
                values.iter().for_each(|&value| vec.push(value));
                m.cur.extend(values.into_iter().map(Some));
            }
            40..=45 if len > 0 => {
                let at = if rng.chance(6) {
                    rng.below(len + 1)
                } else {
                    len - rng.below(len.min(300) + 1)
                };
                vec.truncate_if_needed_at(at).unwrap();
                m.truncated |= at < len;
                m.cur.truncate(at);
            }
            46..=53 if V::MUTABLE && len > 0 => {
                // Sometimes only stored zeros, with that previous value given for their undo.
                let zeros: Vec<usize> = (0..vec.stored_len())
                    .filter(|&i| m.cur[i] == Some(0))
                    .collect();
                let previous = (!zeros.is_empty() && rng.chance(2)).then_some(0);
                let most = if rng.chance(4) { 2_000 } else { 16 };
                let count = 1 + rng.below(most);
                let updates: BTreeMap<usize, u64> = values(rng, count)
                    .into_iter()
                    .map(|value| match previous {
                        Some(_) => (zeros[rng.below(zeros.len())], value),
                        None => (rng.below(len), value),
                    })
                    .collect();
                updates
                    .iter()
                    .for_each(|(&i, &value)| m.cur[i] = Some(value));
                vec.update(updates.into_iter().collect(), previous);
            }
            54..=56 if V::MUTABLE && len > 0 => {
                let indexes: BTreeSet<usize> =
                    (0..1 + rng.below(16)).map(|_| rng.below(len)).collect();
                indexes.iter().for_each(|&i| m.cur[i] = None);
                vec.delete(&indexes.into_iter().collect::<Vec<_>>());
            }
            57..=59 if V::MUTABLE => {
                let count = 1 + rng.below(16);
                let values = values(rng, count);
                let indexes: Vec<usize> = (0..len)
                    .filter(|&i| m.cur[i].is_none())
                    .chain(len..)
                    .take(values.len())
                    .collect();
                assert_eq!(vec.fill(values.clone()), indexes, "lowest holes first");
                for (i, value) in indexes.into_iter().zip(values) {
                    match m.cur.get_mut(i) {
                        Some(slot) => *slot = Some(value),
                        None => m.cur.push(Some(value)),
                    }
                }
            }
            60..=71 => checkpoint(vec, m, rng),
            // A plain write would cut rows the next undo record must read back.
            72..=76 if !m.truncated => {
                match rng.below(3) {
                    0 => {
                        vec.write().unwrap();
                    }
                    1 => vec.flush().unwrap(),
                    _ => vec.stamped_write(Stamp::from(m.stamp)).unwrap(),
                }
                m.written();
                assert!(!vec.is_dirty());
            }
            77..=79 => {
                let target = (m.stamp + 1).saturating_sub(rng.below(8) as u64).max(1);
                let landing = m.landing(target);
                let planned = vec.rollback_landing(Stamp::from(target)).unwrap();
                assert_eq!(planned, landing.map(Stamp::from));
                if let Some(landing) = landing {
                    let landed = vec.rollback_before(Stamp::from(target)).unwrap();
                    assert_eq!(landed, Stamp::from(landing));
                    m.rollback_to(landing);
                } else if rng.chance(4) {
                    // The caller rebuilds.
                    vec.reset().unwrap();
                    m.reset();
                }
            }
            80 => {
                if let Some(landing) = m.landing(m.stamp) {
                    vec.rollback().unwrap();
                    m.rollback_to(landing);
                }
            }
            81..=83 => {
                vec.reset_unsaved();
                m.cur = m.cancel.clone();
                m.truncated = false;
            }
            84 if rng.chance(2) => {
                let dependency = 1 + u32::from(rng.chance(6));
                let changed = m.dependency != Some(dependency);
                let reset = vec
                    .any_validate_computed_version_or_reset(Version::new(dependency))
                    .unwrap();
                assert_eq!(reset, changed);
                if changed {
                    m.dependency = Some(dependency);
                    m.reset();
                }
            }
            85 => {
                vec.reset().unwrap();
                m.reset();
            }
            _ => {}
        }
    }

    fn check(&self, rng: &mut Rng) {
        let m = &self.state.model;
        assert_eq!(self.vec.stamp(), Stamp::from(m.stamp));
        check_reads(&self.vec, &m.cur, rng);
        self.vec.check_own(&m.cur, rng);
        // The unsaved values follow the stored ones (a hole's slot holds anything).
        let (stored, pushed) = (self.vec.stored_len(), self.vec.pushed());
        assert_eq!(stored + pushed.len(), m.cur.len(), "pushed length");
        let mut unsaved = pushed.iter().zip(&m.cur[stored..]);
        assert!(
            unsaved.all(|(value, expect)| expect.is_none_or(|x| *value == x)),
            "pushed values"
        );
        if self.vec.is_dirty() {
            return;
        }
        // Written: the header on disk too, and clones (one kept across writes, one fresh and
        // erased) see it all.
        assert_eq!(m.cur, m.disk);
        let header = Header::read_unverified(self.vec.region()).unwrap();
        assert_eq!(
            (header.stamp(), header.computed_version()),
            (self.vec.stamp(), self.vec.header().computed_version()),
            "header on disk"
        );
        check_reads(&self.clone, &m.cur, rng);
        V::check_clone(&self.clone, &m.cur, rng);
        let boxed = self.vec.read_only_boxed_clone();
        assert_eq!(boxed.is_mutable(), V::MUTABLE);
        check_reads(&boxed, &m.cur, rng);
        let lazy = LazyVec::init("lazy", Version::ONE, boxed.clone(), lazy_value);
        assert_eq!(lazy.is_mutable(), V::MUTABLE);
        if V::MUTABLE {
            return;
        }
        // Views over a dense source: a transform, and a rolling sum over moving windows.
        let x: Vec<u64> = m.cur.iter().map(|value| value.unwrap()).collect();
        let mapped: Vec<_> = (0..x.len()).map(|i| Some(lazy_value(i, x[i]))).collect();
        check_reads(&lazy, &mapped, rng);
        let starts = LazyVec::<usize, usize, usize, u64>::init(
            "starts",
            Version::ONE,
            boxed.clone(),
            |i, _| window_start(i),
        );
        let sums = LazyDeltaVec::<usize, u64, u64, DeltaSub>::new(
            "sums",
            Version::ONE,
            boxed,
            starts.into_read_only_boxed(),
        );
        let expect: Vec<_> = (0..x.len())
            .map(|h| {
                let before = window_start(h).checked_sub(1).map_or(0, |i| x[i]);
                Some(x[h].saturating_sub(before))
            })
            .collect();
        check_reads(&sums, &expect, rng);
    }
}

/// A checkpoint at a later stamp, mostly saving its undo record.
fn checkpoint<V: Variant>(vec: &mut V, m: &mut Model, rng: &mut Rng) {
    let stamp = m.stamp + 1 + rng.below(3) as u64;
    let with_changes = !rng.chance(6);
    vec.stamped_write_maybe_with_changes(Stamp::from(stamp), with_changes)
        .unwrap();
    m.checkpoint(stamp, with_changes);
    assert!(!vec.is_dirty());
}

/// A vector's expected state (`None` for a hole).
#[derive(Default)]
struct Model {
    cur: Vec<Option<u64>>,
    /// What cancelling unsaved changes returns to.
    cancel: Vec<Option<u64>>,
    /// The state at the last checkpoint or rollback: the next undo record returns to it.
    base: Vec<Option<u64>>,
    disk: Vec<Option<u64>>,
    /// The base of what is on disk: a plain write since a checkpoint makes them differ.
    disk_base: Vec<Option<u64>>,
    stamp: u64,
    disk_stamp: u64,
    dependency: Option<u32>,
    disk_dependency: Option<u32>,
    /// Undo records by stamp: the stamp they land on and the state they restore.
    undo: BTreeMap<u64, (u64, Vec<Option<u64>>)>,
    /// A rollback's or a reset's state is not written yet.
    unwritten: bool,
    /// Truncated since the last checkpoint or rollback.
    truncated: bool,
}

impl Model {
    fn written(&mut self) {
        if self.unwritten {
            self.disk_base = self.base.clone();
        }
        self.disk = self.cur.clone();
        self.cancel = self.cur.clone();
        self.disk_stamp = self.stamp;
        self.disk_dependency = self.dependency;
        self.unwritten = false;
    }

    /// Saves the undo record at `stamp` (keeping it and the newest older ones, dropping newer
    /// ones), or without changes drops the records above the current stamp.
    fn checkpoint(&mut self, stamp: u64, with_changes: bool) {
        if with_changes {
            self.undo.insert(stamp, (self.stamp, self.base.clone()));
            self.undo.retain(|&saved, _| saved <= stamp);
            while self.undo.len() > usize::from(KEPT) {
                self.undo.pop_first();
            }
        } else {
            let current = self.stamp;
            self.undo.retain(|&saved, _| saved <= current);
        }
        self.stamp = stamp;
        self.written();
        self.base = self.cur.clone();
        self.disk_base = self.cur.clone();
        self.truncated = false;
    }

    /// Where undoing every record down below `target` lands, if the chain reaches.
    fn landing(&self, target: u64) -> Option<u64> {
        let mut at = self.stamp;
        while at >= target {
            at = self.undo.get(&at)?.0;
        }
        Some(at)
    }

    fn rollback_to(&mut self, landing: u64) {
        if landing == self.stamp {
            // Nothing to undo: unsaved changes stay.
            return;
        }
        while self.stamp > landing {
            let (previous, state) = self.undo[&self.stamp].clone();
            self.stamp = previous;
            self.cur = state;
        }
        self.base = self.cur.clone();
        self.cancel = self.cur.clone();
        self.unwritten = true;
        self.truncated = false;
    }

    /// Empty until written; the undo records are gone at once.
    fn reset(&mut self) {
        self.cur.clear();
        self.cancel.clear();
        self.base.clear();
        self.stamp = 0;
        self.undo.clear();
        self.unwritten = true;
        self.truncated = false;
    }

    /// What an import finds: the written state, or nothing under a new version.
    fn reopen(&mut self, new_version: bool) {
        if new_version {
            *self = Self::default();
            return;
        }
        self.cur = self.disk.clone();
        self.cancel = self.disk.clone();
        self.base = self.disk.clone();
        self.stamp = self.disk_stamp;
        self.dependency = self.disk_dependency;
        self.unwritten = false;
        self.truncated = false;
    }
}

/// Every read API of `v` against `expect` (`None` for a hole): length, a random range through
/// each range read, points, sorted indexes, the retained cache and, on dense values, cursors.
fn check_reads<R: ReadableVec<usize, u64>>(v: &R, expect: &[Option<u64>], rng: &mut Rng) {
    let what = type_name::<R>();
    let len = expect.len();
    assert_eq!(v.len(), len, "{what}: length");
    if rng.chance(8) {
        assert_eq!(v.collect(), present(expect, 0, len), "{what}: all");
    }
    let from = rng.below(len + 2);
    let to = from + rng.below(3_000);
    let want = present(expect, from, to);
    assert_eq!(v.collect_range_at(from, to), want, "{what}: {from}..{to}");
    let mut appended = vec![u64::MAX];
    v.read_into_at(from, to, &mut appended);
    assert_eq!(
        (appended[0], &appended[1..]),
        (u64::MAX, &want[..]),
        "{what}"
    );
    let mut visited = Vec::new();
    v.for_each_range_dyn_at(from, to, &mut |value| visited.push(value));
    assert_eq!(visited, want, "{what}: visit");
    let mut chunks = Vec::new();
    v.for_each_chunk_at(from, to, &mut |at, chunk| {
        assert!(
            !chunk.is_empty() && at == from + chunks.len(),
            "{what}: chunk at {at}"
        );
        chunks.extend_from_slice(chunk);
    });
    assert_eq!(chunks, want, "{what}: chunks");
    let folded = v.fold_range_at(from, to, Vec::new(), |mut values, value| {
        values.push(value);
        values
    });
    assert_eq!(folded, want, "{what}: fold");
    let stop = rng.below(want.len() + 1);
    let stopped = v.try_fold_range_at(from, to, 0, |count, value| {
        if count == stop {
            Err(value)
        } else {
            Ok(count + 1)
        }
    });
    let expected = want.get(stop).map_or(Ok(want.len()), |&value| Err(value));
    assert_eq!(stopped, expected, "{what}: early exit");
    let mut cached = Vec::new();
    if v.read_cached_into_at(from, to, &mut cached) {
        assert_eq!(cached, want, "{what}: cached");
    } else {
        assert!(cached.is_empty(), "{what}: a cache miss appended");
    }

    for _ in 0..4 {
        let i = rng.below(len + 2);
        assert_eq!(v.collect_one_at(i), get(expect, i), "{what}: at {i}");
    }
    assert_eq!(v.collect_last(), expect.last().copied().flatten(), "{what}");
    let mut indexes: Vec<usize> = if rng.chance(2) {
        let start = rng.below(len + 1);
        (start..start + rng.below(64)).collect()
    } else {
        (0..rng.below(64)).map(|_| rng.below(len + 2)).collect()
    };
    indexes.sort_unstable();
    let sorted: Vec<u64> = indexes.iter().filter_map(|&i| get(expect, i)).collect();
    assert_eq!(v.read_sorted_at(&indexes), sorted, "{what}: {indexes:?}");

    // Cursors pair buffer offsets with positions: dense sources only.
    if expect.iter().any(Option::is_none) {
        return;
    }
    let mut cursor = v.cursor();
    for _ in 0..4 {
        let i = rng.below(len + 2);
        assert_eq!(cursor.get(i), get(expect, i), "{what}: cursor at {i}");
    }
    let mut range = Vec::new();
    cursor.collect_range_into_at(from, to, &mut range);
    assert_eq!(range, want, "{what}: cursor range");
    range.clear();
    cursor
        .try_for_each_range_at::<()>(from, to, |value| {
            range.push(value);
            Ok(())
        })
        .unwrap();
    assert_eq!(range, want, "{what}: cursor visit");
    let mut bounded = v.bounded_cursor_at(from, to);
    let (head, skip) = (rng.below(want.len() + 1), rng.below(64));
    let mut seen: Vec<u64> = (0..head).map(|_| bounded.next().unwrap()).collect();
    bounded.advance(skip);
    while let Some(value) = bounded.next() {
        seen.push(value);
    }
    let rest = &want[(head + skip).min(want.len())..];
    assert_eq!(
        seen,
        [&want[..head], rest].concat(),
        "{what}: bounded cursor"
    );
}

/// A persisted range cursor (raw or compressed) against `want`: values one by one, a skip,
/// then the rest in bulk.
macro_rules! check_range_cursor {
    ($cursor:expr, $want:expr, $rng:expr) => {{
        let (mut cursor, want, rng): (_, &[u64], &mut Rng) = ($cursor, $want, $rng);
        let (head, skip) = (rng.below(want.len() + 1), rng.below(64));
        let mut seen: Vec<u64> = (0..head).map(|_| cursor.next().unwrap()).collect();
        cursor.advance(skip);
        cursor.for_each(usize::MAX, |value| seen.push(value));
        assert_eq!(cursor.next(), None);
        let rest = &want[(head + skip).min(want.len())..];
        assert_eq!(seen, [&want[..head], rest].concat(), "range cursor");
    }};
}

/// What the harness needs from a variant beyond [`StoredVec`]: in-place mutations (mutable
/// variants only) and checks of the variant's own readers.
trait Variant: StoredVec<I = usize, T = u64> {
    const MUTABLE: bool;

    /// Replaces the values at distinct indexes; with `previous`, the stored ones all equal it.
    fn update(&mut self, _updates: Vec<(usize, u64)>, _previous: Option<u64>) {
        unreachable!("append-only")
    }

    fn delete(&mut self, _indexes: &[usize]) {
        unreachable!("append-only")
    }

    /// Fills the lowest holes, then appends: returns the indexes written.
    fn fill(&mut self, _values: Vec<u64>) -> Vec<usize> {
        unreachable!("append-only")
    }

    fn check_own(&self, expect: &[Option<u64>], rng: &mut Rng);

    /// Checks the persisted readers of a written vector's read-only clone.
    fn check_clone(_clone: &Self::ReadOnly, _expect: &[Option<u64>], _rng: &mut Rng) {}
}

impl<S: RawStrategy<u64>, C: CachePolicy> Variant for ReadWriteRawVec<usize, u64, S, C> {
    const MUTABLE: bool = false;

    fn check_own(&self, expect: &[Option<u64>], rng: &mut Rng) {
        let reader = self.reader();
        for _ in 0..4 {
            let i = rng.below(expect.len() + 2);
            assert_eq!(self.get_append_only(i, &reader), get(expect, i));
        }
        drop(reader);
        // Rows below the stored length are on disk as the model has them: append-only.
        let stored = &expect[..self.stored_len()];
        let from = rng.below(stored.len() + 2);
        let to = from + rng.below(3_000);
        check_range_cursor!(
            self.range_cursor_at(from, to),
            &present(stored, from, to),
            rng
        );
    }

    fn check_clone(clone: &Self::ReadOnly, expect: &[Option<u64>], rng: &mut Rng) {
        let reader = clone.reader();
        for _ in 0..4 {
            let i = rng.below(expect.len() + 2);
            assert_eq!(reader.try_get_at(i), get(expect, i));
        }
    }
}

impl<S: CompressionStrategy<u64>, C: CachePolicy> Variant
    for ReadWriteCompressedVec<usize, u64, S, C>
{
    const MUTABLE: bool = false;

    fn check_own(&self, expect: &[Option<u64>], rng: &mut Rng) {
        let stored = &expect[..self.stored_len()];
        let from = rng.below(stored.len() + 2);
        let to = from + rng.below(3_000);
        check_range_cursor!(
            self.range_cursor_at(from, to),
            &present(stored, from, to),
            rng
        );
    }
}

impl Variant for MutableVec<BytesVec<usize, u64>> {
    const MUTABLE: bool = true;

    fn update(&mut self, updates: Vec<(usize, u64)>, previous: Option<u64>) {
        match previous {
            Some(previous) => self.update_many_with_previous(updates, previous),
            None => self.update_many(updates),
        }
        .unwrap();
    }

    fn delete(&mut self, indexes: &[usize]) {
        indexes.iter().for_each(|&i| self.delete_at(i));
    }

    fn fill(&mut self, values: Vec<u64>) -> Vec<usize> {
        self.fill_holes_or_push_many(values.into_iter())
    }

    fn check_own(&self, expect: &[Option<u64>], rng: &mut Rng) {
        let reader = self.reader();
        for _ in 0..8 {
            let i = rng.below(expect.len() + 2);
            assert_eq!(self.get_with_reader(i, &reader), get(expect, i));
        }
    }

    /// The raw reader sees what a hole's slot holds: holes are skipped.
    fn check_clone(clone: &Self::ReadOnly, expect: &[Option<u64>], rng: &mut Rng) {
        let reader = clone.reader();
        for _ in 0..4 {
            let i = rng.below(expect.len() + 2);
            if expect.get(i).is_none_or(Option::is_some) {
                assert_eq!(reader.try_get_at(i), get(expect, i));
            }
        }
    }
}

impl Variant for OverflowVec<usize, u64> {
    const MUTABLE: bool = true;

    fn update(&mut self, updates: Vec<(usize, u64)>, _previous: Option<u64>) {
        self.update_many(updates).unwrap();
    }

    fn delete(&mut self, indexes: &[usize]) {
        self.delete_many(indexes.iter().copied());
    }

    fn fill(&mut self, values: Vec<u64>) -> Vec<usize> {
        self.fill_holes_or_push_many(values)
    }

    fn check_own(&self, expect: &[Option<u64>], rng: &mut Rng) {
        let reader = self.reader();
        for _ in 0..8 {
            let i = rng.below(expect.len() + 2);
            assert_eq!(self.get_with_reader(i, &reader), get(expect, i));
        }
    }

    /// A hole's pointer may name a freed sidecar slot: holes are skipped.
    fn check_clone(clone: &Self::ReadOnly, expect: &[Option<u64>], rng: &mut Rng) {
        let reader = clone.reader();
        for _ in 0..4 {
            let i = rng.below(expect.len() + 2);
            if expect.get(i).is_none_or(Option::is_some) {
                assert_eq!(reader.try_get(i), get(expect, i));
            }
        }
    }
}

/// Inline below 2^31, else a pointer into the sidecar (the top bit set).
impl OverflowVecValue for u64 {
    type Compact = u32;

    const VERSION: Version = Version::ONE;

    fn to_compact(&self) -> Option<u32> {
        u32::try_from(*self).ok().filter(|&value| value < 1 << 31)
    }

    fn from_compact(compact: u32) -> Self {
        u64::from(compact)
    }

    fn overflow_index(compact: u32) -> Option<usize> {
        (compact >= 1 << 31).then(|| (compact - (1 << 31)) as usize)
    }

    fn from_overflow_index(index: usize) -> u32 {
        1 << 31 | u32::try_from(index).unwrap()
    }
}

/// Readers on two threads (the read-only clone and an erased one) see whole values at their
/// indexes while the writer appends, rewrites the tail and, on mutable variants, updates and
/// deletes in place.
fn concurrent<V: Variant>(db: &Database, rng: &mut Rng) {
    let options =
        ImportOptions::new(db, "concurrent", Version::ONE).with_saved_stamped_changes(KEPT);
    let mut vec = V::import_with(options).unwrap();
    let (clone, boxed) = (vec.read_only_clone(), vec.read_only_boxed_clone());
    let mut expect = Vec::new();
    let done = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| read_whole(&clone, !V::MUTABLE, Rng(3), &done));
        scope.spawn(|| read_whole(&boxed, !V::MUTABLE, Rng(5), &done));
        let _stop = Stop(&done);
        for generation in 1..=400 {
            let len = expect.len();
            let truncated = rng.chance(4);
            if truncated {
                let at = len - rng.below(len.min(500) + 1);
                vec.truncate_if_needed_at(at).unwrap();
                expect.truncate(at);
            }
            if V::MUTABLE && !expect.is_empty() {
                // Near the tail, where the readers read: freed sidecar slots are reused at once.
                let len = expect.len();
                let updates: BTreeMap<usize, u64> = (0..rng.below(64))
                    .map(|_| len - 1 - rng.below(len.min(2_000)))
                    .map(|i| (i, whole(i, generation)))
                    .collect();
                updates
                    .iter()
                    .for_each(|(&i, &value)| expect[i] = Some(value));
                vec.update(updates.into_iter().collect(), None);
                let deleted: BTreeSet<usize> = (0..rng.below(16))
                    .map(|_| len - 1 - rng.below(len.min(2_000)))
                    .collect();
                deleted.iter().for_each(|&i| expect[i] = None);
                vec.delete(&deleted.into_iter().collect::<Vec<_>>());
            }
            for i in expect.len()..expect.len() + rng.below(600) {
                vec.push(whole(i, generation));
                expect.push(Some(whole(i, generation)));
            }
            // A plain write would cut rows the next undo record must read back.
            if truncated || rng.chance(2) {
                vec.stamped_write_maybe_with_changes(Stamp::from(generation), rng.chance(2))
                    .unwrap();
            } else {
                vec.write().unwrap();
            }
        }
    });
    check_reads(&vec, &expect, rng);
    check_reads(&clone, &expect, rng);
}

/// Reads near the tail until `done`: each value whole, at its own index (in order, contiguous
/// from where a range read starts on dense variants, exact on point reads) and never older than
/// one this reader saw there before.
fn read_whole<R: ReadableVec<usize, u64>>(v: &R, dense: bool, mut rng: Rng, done: &AtomicBool) {
    let mut seen = Vec::new();
    while !done.load(Relaxed) {
        let len = v.len();
        let from = len.saturating_sub(rng.below(5_000));
        let step = 1 + rng.below(50);
        let mut values = Vec::new();
        let ranged = match rng.below(3) {
            0 => {
                values = v.collect_range_at(from, len + 64);
                true
            }
            1 => {
                v.for_each_chunk_at(from, len + 64, &mut |_, chunk| {
                    values.extend_from_slice(chunk)
                });
                true
            }
            _ => {
                let indexes: Vec<usize> = (from..len + 64).step_by(step).collect();
                values = v.read_sorted_at(&indexes);
                false
            }
        };
        let parts: Vec<(usize, u64)> = values.into_iter().map(whole_parts).collect();
        assert!(
            parts.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "out of order"
        );
        for (k, &(i, generation)) in parts.iter().enumerate() {
            assert!(i >= from, "{i} read from {from}");
            if ranged && dense {
                assert_eq!(i, from + k, "misplaced value");
            } else if !ranged {
                assert_eq!((i - from) % step, 0, "unrequested value");
            }
            fresh(&mut seen, i, generation);
        }
        // Point reads are exact, holes or not.
        let at = from + rng.below(len + 1 - from);
        for i in at..at + 32 {
            if let Some(value) = v.collect_one_at(i) {
                let (index, generation) = whole_parts(value);
                assert_eq!(index, i, "misplaced value");
                fresh(&mut seen, i, generation);
            }
        }
        // Gaps between reads, as between requests: two readers that never pause starve a
        // budgeted source's writer (its cache gate lets readers past a waiting writer: accepted).
        thread::yield_now();
    }
}

/// Publication only moves forward: no value at `index` older than one seen there (small values
/// carry no generation).
fn fresh(seen: &mut Vec<u64>, index: usize, generation: u64) {
    if generation == 0 {
        return;
    }
    if seen.len() <= index {
        seen.resize(index + 1, 0);
    }
    assert!(generation >= seen[index], "stale value at {index}");
    seen[index] = generation;
}

/// A value naming its index and generation, recoverable only from an untorn read: big values
/// hold the generation in the high half and the index mixed with it in the low half. Every third
/// generation is small: the index alone, inline in an overflow vector.
fn whole(index: usize, generation: u64) -> u64 {
    if generation.is_multiple_of(3) {
        return index as u64;
    }
    generation << 32 | (index as u64 ^ mix(generation))
}

/// The index and generation (0 for small values) of a [`whole`] value.
fn whole_parts(value: u64) -> (usize, u64) {
    let generation = value >> 32;
    let low = value & 0xFFFF_FFFF;
    if generation == 0 {
        return (low as usize, 0);
    }
    ((low ^ mix(generation)) as usize, generation)
}

fn mix(generation: u64) -> u64 {
    generation.wrapping_mul(0x9E37_79B9) & 0xFFFF_FFFF
}

/// Stops the readers when the writer finishes or panics, so the scope can end.
struct Stop<'a>(&'a AtomicBool);

impl Drop for Stop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Relaxed);
    }
}

/// The values of `expect[from..to]` (clamped), holes skipped.
fn present(expect: &[Option<u64>], from: usize, to: usize) -> Vec<u64> {
    let len = expect.len();
    expect[from.min(len)..to.min(len)]
        .iter()
        .flatten()
        .copied()
        .collect()
}

fn get(expect: &[Option<u64>], i: usize) -> Option<u64> {
    expect.get(i).copied().flatten()
}

/// Values of one random shape: zeros, small numbers, a run, any width, or a constant, so
/// compressors meet every kind of page and overflow values both fit inline and spill.
fn values(rng: &mut Rng, count: usize) -> Vec<u64> {
    let (shape, start) = (rng.below(5), rng.next());
    (0..count as u64)
        .map(|i| match shape {
            0 => 0,
            1 => rng.next() % 1_000,
            2 => start.wrapping_add(i * 3),
            3 => rng.next() >> rng.below(64),
            _ => start,
        })
        .collect()
}

fn lazy_value(index: usize, value: u64) -> u64 {
    value.rotate_left(17) ^ index as u64
}

/// Window starts that move back and forth: near the index, or halfway back.
fn window_start(index: usize) -> usize {
    if index.is_multiple_of(7) {
        index / 2
    } else {
        index.saturating_sub(13)
    }
}

/// xorshift64: deterministic, no dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// In `0..n`; 0 when `n` is 0.
    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }

    fn chance(&mut self, one_in: usize) -> bool {
        self.below(one_in) == 0
    }
}

/// A directory removed when dropped, after a panic too.
struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
