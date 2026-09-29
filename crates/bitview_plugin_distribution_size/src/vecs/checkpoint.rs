use bitview_cohort::{AmountRangeId, SpendableTypeId};
use brk_error::Result;
use brk_types::{Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, Stamp, WritableVec};

use crate::{
    Vecs,
    state::{AddrStates, MinimalRealizedState, RealizedOps, UTXOStates},
};

// Stable order: UTXO amounts, output types, address amounts. Bump the vector
// version when changing this layout. This is one 656-byte state, not a time series.
const CAP_COUNT: usize = 2 * AmountRangeId::ALL.len() + SpendableTypeId::ALL.len();

impl Vecs {
    pub(crate) fn cap_checkpoint_len(&self) -> usize {
        if self.caps.len() == CAP_COUNT {
            usize::from(Height::from(self.caps.stamp()).incremented())
        } else {
            0
        }
    }

    pub(crate) fn validate_state(&mut self, version: Version) -> Result<bool> {
        self.addr_state
            .par_iter_mut()
            .chain([&mut self.caps as &mut dyn AnyStoredVec].into_par_iter())
            .map(|v| {
                let changed = v.header().computed_version() != v.header().vec_version() + version;
                v.any_validate_computed_version_or_reset(version)?;
                Ok(changed)
            })
            .try_reduce(|| false, |a, b| Ok(a || b))
    }

    pub(crate) fn rollback_state(&mut self, start: usize) -> Result<usize> {
        let stamp = Stamp::from(Height::from(start));
        let mut stamps = self.addr_state.rollback_before(stamp)?;
        stamps.push(self.caps.rollback_before(stamp)?);
        let recovered = usize::from(Height::from(stamps[0]).incremented());
        Ok(
            if recovered <= start && stamps.iter().all(|s| *s == stamps[0]) {
                recovered
            } else {
                0
            },
        )
    }

    pub(crate) fn restore_caps(
        &self,
        utxos: &mut UTXOStates,
        addrs: &mut AddrStates,
    ) -> Option<()> {
        let caps = self.caps.collect();
        if caps.len() != CAP_COUNT {
            return None;
        }
        for (state, cap) in utxos
            .amount_range
            .iter_mut()
            .chain(utxos.type_.iter_mut())
            .map(|s| &mut s.realized)
            .chain(addrs.amount_range.iter_mut().map(|s| &mut s.inner.realized))
            .zip(caps)
        {
            *state = MinimalRealizedState::from_cap(cap);
        }
        Some(())
    }

    pub(crate) fn save_caps(
        &mut self,
        utxos: &UTXOStates,
        addrs: &AddrStates,
        stamp: Stamp,
        with_changes: bool,
    ) -> Result<()> {
        let caps = utxos
            .amount_range
            .iter()
            .chain(utxos.type_.iter())
            .map(|s| s.realized.cap_raw())
            .chain(
                addrs
                    .amount_range
                    .iter()
                    .map(|s| s.inner.realized.cap_raw()),
            );
        if self.caps.is_empty() {
            self.caps.extend(caps);
        } else {
            self.caps.update_many(caps.enumerate())?;
        }
        self.caps
            .stamped_write_maybe_with_changes(stamp, with_changes)?;
        Ok(())
    }
}
