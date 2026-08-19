use brk_error::Result;

use std::{
    collections::{BTreeMap, btree_map::Entry},
    fs,
    path::Path,
};

use brk_error::Error;
use brk_types::{Cents, CentsCompact, CentsSats, CentsSquaredSats, Height, Sats, UrpdRaw};
use rustc_hash::FxHashMap;
use vecdb::{Bytes, unlikely};

use super::unrealized::CachedUnrealizedState;
use super::{Accumulate, CostBasisOps, CostBasisRaw, UnrealizedState};
use crate::state::pending::{PendingCapitalizedCapRawDelta, PendingDelta};

/// Full cost basis tracking: BTreeMap distribution + raw scalars.
/// Composes `CostBasisRaw` for scalar tracking, adds map, pending, and cache.
///
/// Generic over the accumulator `S`:
/// - `WithCapital`: tracks all fields including invested capital + capitalized cap (128 bytes)
/// - `WithoutCapital`: tracks only supply + unrealized profit/loss (64 bytes, 1 cache line)
///   and writes a compact checkpoint without a capitalized-cap slot
#[derive(Clone, Debug)]
pub struct CostBasisData<S: Accumulate> {
    raw: CostBasisRaw,
    map: Option<UrpdRaw>,
    pending: FxHashMap<CentsCompact, PendingDelta>,
    cache: Option<CachedUnrealizedState<S>>,
    capitalized_cap_raw: CentsSquaredSats,
    pending_capitalized_cap: PendingCapitalizedCapRawDelta,
}

impl<S: Accumulate> CostBasisData<S> {
    pub fn map(&self) -> &BTreeMap<CentsCompact, Sats> {
        debug_assert!(self.pending.is_empty() && self.raw.has_no_pending_cap());
        &self.map.as_ref().unwrap().map
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.map.as_ref().unwrap().map.is_empty()
    }

    pub fn for_each_pending(&self, mut f: impl FnMut(&CentsCompact, &PendingDelta)) {
        self.pending.iter().for_each(|(k, v)| f(k, v));
    }

    pub fn compute_unrealized_state(&mut self, height_price: Cents) -> UnrealizedState {
        if self.is_empty() {
            return UnrealizedState::ZERO;
        }

        let map = &self.map.as_ref().unwrap().map;

        if let Some(cache) = self.cache.as_mut() {
            cache.get_at_price(height_price, map)
        } else {
            let cache = CachedUnrealizedState::compute_fresh(height_price, map);
            let state = cache.current_state();
            self.cache = Some(cache);
            state
        }
    }

    fn apply_map_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let map = &mut self.map.as_mut().unwrap().map;
        for (cents, PendingDelta { inc, dec }) in self.pending.drain() {
            match map.entry(cents) {
                Entry::Occupied(mut e) => {
                    *e.get_mut() += inc;
                    if unlikely(*e.get() < dec) {
                        panic!(
                            "CostBasisData::apply_pending underflow!\n\
                            Path: {:?}\n\
                            Price: {}\n\
                            Current + increments: {}\n\
                            Trying to decrement by: {}",
                            self.raw.path(),
                            cents.to_dollars(),
                            e.get(),
                            dec
                        );
                    }
                    *e.get_mut() -= dec;
                    if *e.get() == Sats::ZERO {
                        e.remove();
                    }
                }
                Entry::Vacant(e) => {
                    if unlikely(inc < dec) {
                        panic!(
                            "CostBasisData::apply_pending underflow (new entry)!\n\
                            Path: {:?}\n\
                            Price: {}\n\
                            Increment: {}\n\
                            Trying to decrement by: {}",
                            self.raw.path(),
                            cents.to_dollars(),
                            inc,
                            dec
                        );
                    }
                    let val = inc - dec;
                    if val != Sats::ZERO {
                        e.insert(val);
                    }
                }
            }
        }
    }
}

impl<S: Accumulate> CostBasisOps for CostBasisData<S> {
    fn create(path: &Path, name: &str) -> Self {
        Self {
            raw: CostBasisRaw::create(path, name),
            map: None,
            pending: FxHashMap::default(),
            cache: None,
            capitalized_cap_raw: CentsSquaredSats::ZERO,
            pending_capitalized_cap: PendingCapitalizedCapRawDelta::default(),
        }
    }

    fn import_at_or_before(&mut self, height: Height) -> Result<Height> {
        let files = self.raw.read_dir(None)?;
        let (&height, path) = files.range(..=height).next_back().ok_or(Error::NotFound(
            "No cost basis state found at or before height".into(),
        ))?;
        let data = fs::read(path)?;
        let (base, rest) = UrpdRaw::deserialize_with_rest(&data)?;
        debug_assert!(
            rest.len() >= if S::TRACK_CAPITAL { 32 } else { 16 },
            "CostBasisData state too short: {} bytes",
            rest.len()
        );
        self.map = Some(base);
        self.raw.import_state(rest)?;
        self.capitalized_cap_raw = if S::TRACK_CAPITAL {
            CentsSquaredSats::from_bytes(&rest[16..32])?
        } else {
            CentsSquaredSats::ZERO
        };
        self.pending.clear();
        self.pending_capitalized_cap = PendingCapitalizedCapRawDelta::default();
        self.cache = None;
        Ok(height)
    }

    fn cap_raw(&self) -> CentsSats {
        self.raw.cap_raw()
    }

    fn capitalized_cap_raw(&self) -> CentsSquaredSats {
        self.capitalized_cap_raw
    }

    #[inline]
    fn increment(
        &mut self,
        price: Cents,
        sats: Sats,
        price_sats: CentsSats,
        capitalized_cap: CentsSquaredSats,
    ) {
        self.pending.entry(price.into()).or_default().inc += sats;
        self.raw.increment_cap(price_sats);
        if S::TRACK_CAPITAL && capitalized_cap != CentsSquaredSats::ZERO {
            self.pending_capitalized_cap.inc += capitalized_cap;
        }
        if let Some(cache) = self.cache.as_mut() {
            cache.on_receive(price, sats);
        }
    }

    #[inline]
    fn decrement(
        &mut self,
        price: Cents,
        sats: Sats,
        price_sats: CentsSats,
        capitalized_cap: CentsSquaredSats,
    ) {
        self.pending.entry(price.into()).or_default().dec += sats;
        self.raw.decrement_cap(price_sats);
        if S::TRACK_CAPITAL && capitalized_cap != CentsSquaredSats::ZERO {
            self.pending_capitalized_cap.dec += capitalized_cap;
        }
        if let Some(cache) = self.cache.as_mut() {
            cache.on_send(price, sats);
        }
    }

    fn apply_pending(&mut self) {
        self.apply_map_pending();
        self.raw.apply_pending_cap();
        if S::TRACK_CAPITAL {
            self.capitalized_cap_raw += self.pending_capitalized_cap.inc;
            debug_assert!(
                self.capitalized_cap_raw >= self.pending_capitalized_cap.dec,
                "CostBasis capitalized_cap_raw underflow!\n\
                Path: {:?}\n\
                Current (after increments): {:?}\n\
                Trying to decrement by: {:?}",
                self.raw.path(),
                self.capitalized_cap_raw,
                self.pending_capitalized_cap.dec
            );
            self.capitalized_cap_raw -= self.pending_capitalized_cap.dec;
            self.pending_capitalized_cap = PendingCapitalizedCapRawDelta::default();
        }
    }

    fn init(&mut self) {
        self.raw.init();
        self.map.replace(UrpdRaw::default());
        self.pending.clear();
        self.cache = None;
        self.capitalized_cap_raw = CentsSquaredSats::ZERO;
        self.pending_capitalized_cap = PendingCapitalizedCapRawDelta::default();
    }

    fn clean(&mut self) -> Result<()> {
        self.raw.clean()?;
        self.cache = None;
        Ok(())
    }

    fn write(&mut self, height: Height, cleanup: bool) -> Result<()> {
        self.apply_pending();
        self.raw.write_and_cleanup(height, cleanup)?;

        let mut buffer = self.map.as_ref().unwrap().serialize()?;
        buffer.extend(self.raw.serialized_state());
        if S::TRACK_CAPITAL {
            buffer.extend(self.capitalized_cap_raw.to_bytes());
        }
        fs::write(self.raw.path_state(height), buffer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;
    use crate::state::{WithCapital, WithoutCapital};

    static NEXT_PATH_ID: AtomicU64 = AtomicU64::new(0);

    fn test_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "brk-cost-basis-data-{}-{}",
            std::process::id(),
            NEXT_PATH_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn checkpoint_layout_tracks_capital_capability() {
        let root = test_path();

        let mut compact = CostBasisData::<WithoutCapital>::create(&root, "compact");
        compact.clean().unwrap();
        compact.init();
        compact.write(Height::ZERO, false).unwrap();

        let mut full = CostBasisData::<WithCapital>::create(&root, "full");
        full.clean().unwrap();
        full.init();
        full.write(Height::ZERO, false).unwrap();

        let compact_len = fs::metadata(compact.raw.path_state(Height::ZERO))
            .unwrap()
            .len();
        let full_len = fs::metadata(full.raw.path_state(Height::ZERO))
            .unwrap()
            .len();
        assert_eq!(full_len, compact_len + 16);

        let mut compact_reader = CostBasisData::<WithoutCapital>::create(&root, "compact");
        assert_eq!(
            compact_reader.import_at_or_before(Height::ZERO).unwrap(),
            Height::ZERO
        );

        let mut legacy_reader = CostBasisData::<WithoutCapital>::create(&root, "full");
        assert_eq!(
            legacy_reader.import_at_or_before(Height::ZERO).unwrap(),
            Height::ZERO
        );
        assert_eq!(legacy_reader.capitalized_cap_raw(), CentsSquaredSats::ZERO);

        fs::remove_dir_all(root).unwrap();
    }
}
