use std::{
    collections::{BTreeMap, btree_map::Entry},
    mem,
};

use bitview_primitives::CentsCompact;
use brk_types::{Cents, Sats};
use rustc_hash::FxHashMap;
use vecdb::unlikely;

use super::{Accumulate, CostBasisOps, UnrealizedState, unrealized::CachedUnrealizedState};
use crate::state::pending::PendingDelta;

/// Price distribution with deferred changes and cached unrealized analytics.
///
/// Generic over the accumulator `S`:
/// - `WithCapital`: tracks all fields including invested capital + capitalized cap (128 bytes)
/// - `WithoutCapital`: tracks only supply + unrealized profit/loss (64 bytes, 1 cache line)
#[derive(Clone, Debug)]
pub struct CostBasisData<S: Accumulate> {
    map: BTreeMap<CentsCompact, Sats>,
    pending: FxHashMap<CentsCompact, PendingDelta>,
    cache: Option<CachedUnrealizedState<S>>,
}

impl<S: Accumulate> CostBasisData<S> {
    fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.map.is_empty()
    }

    pub(crate) fn compute_unrealized_state(&mut self, height_price: Cents) -> UnrealizedState {
        if self.is_empty() {
            return UnrealizedState::ZERO;
        }

        let map = &self.map;

        if let Some(cache) = self.cache.as_mut() {
            cache.get_at_price(height_price, map)
        } else {
            let cache = CachedUnrealizedState::compute_fresh(height_price, map);
            let state = cache.current_state();
            self.cache = Some(cache);
            state
        }
    }

    /// Bulk-build a compact, sorted price map from restored origin balances.
    pub(crate) fn finish_restore(&mut self) {
        let map = &mut self.map;
        assert!(map.is_empty(), "bulk restore requires an empty price map");
        // Restore sees the entire history; its oversized table must not become
        // the per-block pending map, whose drain scans would retain that capacity.
        let mut entries: Vec<_> = mem::take(&mut self.pending)
            .into_iter()
            .filter_map(|(price, pending)| {
                let sats = pending.inner();
                assert!(sats >= 0, "restored origin balances cannot be negative");
                (sats != 0).then_some((price, Sats::new(sats as u64)))
            })
            .collect();
        entries.sort_unstable_by_key(|entry| entry.0);
        *map = entries.into_iter().collect();
    }
}

impl<S: Accumulate> CostBasisOps for CostBasisData<S> {
    #[inline]
    fn increment(&mut self, price: Cents, sats: Sats) {
        self.pending
            .entry(price.into())
            .or_default()
            .increment(sats);
        if let Some(cache) = self.cache.as_mut() {
            cache.on_receive(price, sats);
        }
    }

    #[inline]
    fn decrement(&mut self, price: Cents, sats: Sats) {
        self.pending
            .entry(price.into())
            .or_default()
            .decrement(sats);
        if let Some(cache) = self.cache.as_mut() {
            cache.on_send(price, sats);
        }
    }

    fn apply_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let map = &mut self.map;
        for (cents, pending) in self.pending.drain() {
            let delta = pending.inner();
            if delta == 0 {
                continue;
            }
            match map.entry(cents) {
                Entry::Occupied(mut e) => {
                    if delta > 0 {
                        *e.get_mut() += Sats::new(delta as u64);
                        continue;
                    }
                    let decrement = Sats::new(delta.unsigned_abs());
                    if unlikely(*e.get() < decrement) {
                        panic!(
                            "CostBasisData::apply_pending underflow!\n\
                            Price: {}\n\
                            Current: {}\n\
                            Trying to decrement by: {}",
                            cents.to_dollars(),
                            e.get(),
                            decrement
                        );
                    }
                    *e.get_mut() -= decrement;
                    if *e.get() == Sats::ZERO {
                        e.remove();
                    }
                }
                Entry::Vacant(e) => {
                    if unlikely(delta < 0) {
                        panic!(
                            "CostBasisData::apply_pending underflow (new entry)!\n\
                            Price: {}\n\
                            Trying to decrement by: {}",
                            cents.to_dollars(),
                            delta.unsigned_abs()
                        );
                    }
                    e.insert(Sats::new(delta as u64));
                }
            }
        }
    }

    fn init(&mut self) {
        self.map.clear();
        self.pending.clear();
        self.cache = None;
    }
}

impl<S: Accumulate> Default for CostBasisData<S> {
    fn default() -> Self {
        Self {
            map: BTreeMap::new(),
            pending: FxHashMap::default(),
            cache: None,
        }
    }
}
