use brk_types::{FundedAddrData, Height, OutputType};

use crate::addr::{AddrReceivePreState, AddrSendPreState, AddrTypeToAddrCount, AddrTypeToSupply};

use super::{AddrTypeToAddrEventCount, ReusedAddrVecs};

/// Runtime state for receive-based (reused) or spend-based (respent)
/// address tracking. Mirrors the persistent fields of [`ReusedAddrVecs`]
/// (funded + total counts, funded supply) plus per-block event counters
/// that reset every block.
///
/// `output_events`, `input_events`, and `active` are cleared via
/// [`Self::reset_per_block`] at the start of each block. The three running
/// totals (`funded`, `total`, `supply`) are recovered from disk at the start
/// of a run via [`From<(&ReusedAddrVecs, Height)>`].
#[derive(Debug, Default)]
pub struct ReusedAddrState {
    pub funded: AddrTypeToAddrCount,
    pub total: AddrTypeToAddrCount,
    pub supply: AddrTypeToSupply,
    pub output_events: AddrTypeToAddrEventCount,
    pub input_events: AddrTypeToAddrEventCount,
    pub active: AddrTypeToAddrEventCount,
}

impl ReusedAddrState {
    #[inline]
    pub fn reset_per_block(&mut self) {
        self.output_events.reset();
        self.input_events.reset();
        self.active.reset();
    }

    /// Apply reused-flavor (receive-based: `funded_txo_count > 1`) updates
    /// for a received output, AFTER the receive has mutated `addr_data`.
    #[inline]
    pub fn on_receive_as_reused(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        let is_now_reused = addr_data.is_reused();

        // Threshold crossing: the 2nd lifetime receive lands here. The address
        // is always funded post-receive.
        if is_now_reused && !pre.was_reused {
            *self.total.get_mut_unwrap(output_type) += 1;
            *self.funded.get_mut_unwrap(output_type) += 1;
        } else if pre.was_reused && !pre.was_funded {
            // Reactivation: already-reused address was empty, now funded.
            *self.funded.get_mut_unwrap(output_type) += 1;
        }

        // output-to-reused events: outputs landing on addresses that had
        // already received >= 1 prior output, i.e. every output except the
        // very first one the address ever sees. With `before =
        // prev_funded_txo_count` and `N = output_count`: events = N - max(0, 1 - before).
        let skip_first = 1u32.saturating_sub(pre.prev_funded_txo_count.min(1));
        let reused_events = output_count.saturating_sub(skip_first);
        if reused_events > 0 {
            *self.output_events.get_mut_unwrap(output_type) += u64::from(reused_events);
        }

        if is_now_reused {
            *self.active.get_mut_unwrap(output_type) += 1;
        }

        let after = addr_data.reused_supply_contribution();
        self.supply
            .apply_delta(output_type, pre.reused_contribution, after);
    }

    /// Apply respent-flavor (spend-based: `spent_txo_count > 1`) updates for a
    /// received output, AFTER the receive has mutated `addr_data`. Receives
    /// don't cross the respent threshold. The only transition is an
    /// already-respent empty address reactivating into the funded set.
    #[inline]
    pub fn on_receive_as_respent(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        if pre.was_respent && !pre.was_funded {
            *self.funded.get_mut_unwrap(output_type) += 1;
        }
        // Respent status is stable across a receive, so every output lands on
        // a respent address iff the address was already respent.
        if pre.was_respent {
            *self.output_events.get_mut_unwrap(output_type) += u64::from(output_count);
            *self.active.get_mut_unwrap(output_type) += 1;
        }
        let after = addr_data.respent_supply_contribution();
        self.supply
            .apply_delta(output_type, pre.respent_contribution, after);
    }

    /// Apply reused-flavor updates for a spent UTXO, AFTER the send has
    /// mutated `addr_data`. Sends don't change the reused predicate, so
    /// `pre.was_reused == is_reused` post-spend.
    #[inline]
    pub fn on_send_as_reused(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if pre.was_reused {
            *self.input_events.get_mut_unwrap(output_type) += 1;
        }
        // Active reused: first-encounter sender, currently reused, and not
        // already counted on the receive side.
        if is_first_encounter && pre.was_reused && !also_received {
            *self.active.get_mut_unwrap(output_type) += 1;
        }
        if will_be_empty && pre.was_reused {
            *self.funded.get_mut_unwrap(output_type) -= 1;
        }
        let after = addr_data.reused_supply_contribution();
        self.supply
            .apply_delta(output_type, pre.reused_contribution, after);
    }

    /// Apply respent-flavor updates for a spent UTXO, AFTER the send has
    /// mutated `addr_data`. Sends CAN cross the respent threshold on the
    /// 2nd lifetime spend.
    #[inline]
    pub fn on_send_as_respent(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if pre.was_respent {
            *self.input_events.get_mut_unwrap(output_type) += 1;
        }

        let is_now_respent = addr_data.is_respent();

        // Threshold crossing: the 2nd spend ever on this address. Always
        // bumps the monotonic total. Bumps the funded count iff the address
        // still has a balance. If the crossing spend also empties the
        // address, the `will_be_empty` branch below doesn't decrement
        // (was_respent is false), so the funded count stays correct.
        if is_now_respent && !pre.was_respent {
            *self.total.get_mut_unwrap(output_type) += 1;
            if !will_be_empty {
                *self.funded.get_mut_unwrap(output_type) += 1;
            }
        }

        // Active respent splits cleanly into two disjoint branches (gated on
        // `pre.was_respent`):
        //   - was already respent + active this block, and not also counted
        //     on the receive side: pure senders on first spend.
        //   - crosses the respent threshold this block: fires once per
        //     address ever, on the exact crossing spend.
        if (is_first_encounter && pre.was_respent && !also_received)
            || (is_now_respent && !pre.was_respent)
        {
            *self.active.get_mut_unwrap(output_type) += 1;
        }

        // Leaving the funded respent set on empty uses pre-spend state: a
        // threshold-crossing spend that also empties the address never
        // entered the funded set above (gated on !will_be_empty), so we
        // don't double-decrement.
        if will_be_empty && pre.was_respent {
            *self.funded.get_mut_unwrap(output_type) -= 1;
        }

        let after = addr_data.respent_supply_contribution();
        self.supply
            .apply_delta(output_type, pre.respent_contribution, after);
    }
}

impl From<(&ReusedAddrVecs, Height)> for ReusedAddrState {
    #[inline]
    fn from((vecs, starting_height): (&ReusedAddrVecs, Height)) -> Self {
        Self {
            funded: AddrTypeToAddrCount::from((&vecs.count.funded, starting_height)),
            total: AddrTypeToAddrCount::from((&vecs.count.total, starting_height)),
            supply: AddrTypeToSupply::from((&vecs.supply, starting_height)),
            output_events: AddrTypeToAddrEventCount::default(),
            input_events: AddrTypeToAddrEventCount::default(),
            active: AddrTypeToAddrEventCount::default(),
        }
    }
}
