use std::{
    collections::{BTreeMap, btree_map::Entry},
    mem,
};

use bitview_primitives::CentsCompact;
use brk_types::{Cents, Sats};
use rustc_hash::FxHashMap;

use super::{CostBasisOps, UnrealizedState, unrealized::CachedUnrealizedState};

/// Price distribution with deferred changes and cached unrealized analytics.
#[derive(Clone, Debug, Default)]
pub struct CostBasisData {
    map: BTreeMap<CentsCompact, Sats>,
    /// Net sats per price since the last apply, wrapping like the signed deltas they are.
    pending: FxHashMap<CentsCompact, i64>,
    cache: Option<CachedUnrealizedState>,
}

impl CostBasisData {
    fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.map.is_empty()
    }

    pub(crate) fn compute_unrealized_state(&mut self, height_price: Cents) -> UnrealizedState {
        if self.is_empty() {
            return UnrealizedState::default();
        }

        let map = &self.map;
        self.cache
            .get_or_insert_with(|| CachedUnrealizedState::compute_fresh(height_price, map))
            .get_at_price(height_price, map)
    }

    /// Bulk-build a compact, sorted price map from restored origin balances.
    pub(crate) fn finish_restore(&mut self) {
        let map = &mut self.map;
        debug_assert!(map.is_empty(), "bulk restore requires an empty price map");
        // Restore sees the entire history; its oversized table must not become
        // the per-block pending map, whose drain scans would retain that capacity.
        let mut entries: Vec<_> = mem::take(&mut self.pending)
            .into_iter()
            .filter_map(|(price, sats)| {
                debug_assert!(sats >= 0, "restored origin balances cannot be negative");
                (sats != 0).then_some((price, Sats::new(sats as u64)))
            })
            .collect();
        entries.sort_unstable_by_key(|entry| entry.0);
        *map = entries.into_iter().collect();
    }
}

impl CostBasisOps for CostBasisData {
    #[inline]
    fn increment(&mut self, price: Cents, sats: Sats) {
        let delta = self.pending.entry(price.into()).or_default();
        *delta = delta.wrapping_add_unsigned(sats.into());
        if let Some(cache) = self.cache.as_mut() {
            cache.on_receive(price, sats);
        }
    }

    #[inline]
    fn decrement(&mut self, price: Cents, sats: Sats) {
        let delta = self.pending.entry(price.into()).or_default();
        *delta = delta.wrapping_sub_unsigned(sats.into());
        if let Some(cache) = self.cache.as_mut() {
            cache.on_send(price, sats);
        }
    }

    fn apply_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let map = &mut self.map;
        for (cents, delta) in self.pending.drain() {
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
                    debug_assert!(
                        *e.get() >= decrement,
                        "price map underflow at {}: {} - {decrement}",
                        cents.to_dollars(),
                        e.get()
                    );
                    *e.get_mut() -= decrement;
                    if *e.get() == Sats::ZERO {
                        e.remove();
                    }
                }
                Entry::Vacant(e) => {
                    debug_assert!(
                        delta > 0,
                        "price map underflow at {}: 0 - {}",
                        cents.to_dollars(),
                        delta.unsigned_abs()
                    );
                    e.insert(Sats::new(delta as u64));
                }
            }
        }
    }
}
