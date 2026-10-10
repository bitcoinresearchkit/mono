use std::ops::AddAssign;

use bitview_primitives::FundedAddrData;
use brk_types::{OutputType, Sats};

use super::{AddrReceivePreState, AddrReceiveStatus, AddrSendPreState, delta::ShardDelta};

/// One address type's running counters (the root sums them): funded and empty counts, the
/// supply the addresses hold, the block's outputs, inputs and activity, and the reuse and
/// exposure predicates.
#[derive(Debug, Default, Clone, Copy)]
pub struct MemberState {
    pub funded: u64,
    pub empty: u64,
    pub supply: Sats,
    /// Per block: outputs the addresses received and inputs they spent.
    pub outputs: u64,
    pub inputs: u64,
    pub activity: BlockActivityCounts,
    pub reused: ReuseState,
    pub respent: ReuseState,
    pub exposed: ExposedState,
}

/// Per-block activity counts, reset after every block.
#[derive(Debug, Default, Clone, Copy)]
pub struct BlockActivityCounts {
    pub reactivated: u32,
    pub sending: u32,
    pub receiving: u32,
    pub bidirectional: u32,
}

/// Addresses meeting a reuse predicate: receive-based reuse or spend-based respending.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReuseState {
    pub funded: u64,
    pub total: u64,
    pub supply: Sats,
    /// Per block: outputs and inputs classified by the predicate, and active addresses meeting
    /// it.
    pub output_events: u64,
    pub input_events: u64,
    pub active: u64,
}

/// Addresses whose public key or spending script is on-chain.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExposedState {
    pub funded: u64,
    pub total: u64,
    pub supply: Sats,
}

impl MemberState {
    #[inline]
    pub fn reset_per_block(&mut self) {
        self.outputs = 0;
        self.inputs = 0;
        self.activity = BlockActivityCounts::default();
        self.reused.reset_per_block();
        self.respent.reset_per_block();
    }

    #[inline]
    pub fn on_receive_applied(
        &mut self,
        output_type: OutputType,
        status: AddrReceiveStatus,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        self.activity.receiving += 1;
        match status {
            AddrReceiveStatus::New => self.funded += 1,
            AddrReceiveStatus::WasEmpty => {
                self.activity.reactivated += 1;
                self.funded += 1;
                self.empty -= 1;
            }
            AddrReceiveStatus::Tracked => {}
        }
        self.reused
            .on_receive_as_reused(addr_data, pre, output_count);
        self.respent
            .on_receive_as_respent(addr_data, pre, output_count);
        self.exposed.on_receive(output_type, addr_data, pre, status);
    }

    /// A received output left the address without a spend, as if never received: the address
    /// stays funded, may no longer count as reused, and its supplies move.
    pub fn on_output_lost(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
    ) {
        debug_assert!(addr_data.is_funded());
        debug_assert!(
            addr_data.is_respent() == pre.was_respent
                && addr_data.is_pubkey_exposed(output_type) == pre.was_pubkey_exposed,
            "forgetting a receive changes no spend-side predicate"
        );
        // The forgotten output was the address's first: the next one, received this block,
        // was no reuse after all.
        if pre.was_reused && !addr_data.is_reused() {
            self.reused.funded -= 1;
            self.reused.total -= 1;
            self.reused.active -= 1;
            self.reused.output_events -= 1;
        }
        apply_supply_delta(
            &mut self.reused.supply,
            pre.reused_contribution,
            addr_data.reused_supply_contribution(),
        );
        apply_supply_delta(
            &mut self.respent.supply,
            pre.respent_contribution,
            addr_data.respent_supply_contribution(),
        );
        apply_supply_delta(
            &mut self.exposed.supply,
            pre.exposed_contribution,
            addr_data.exposed_supply_contribution(output_type),
        );
    }

    #[inline]
    pub fn on_send_applied(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if is_first_encounter {
            self.activity.sending += 1;
            if also_received {
                self.activity.bidirectional += 1;
            }
        }
        if will_be_empty {
            self.funded -= 1;
            self.empty += 1;
        }
        self.reused.on_send_as_reused(
            addr_data,
            pre,
            is_first_encounter,
            also_received,
            will_be_empty,
        );
        self.respent.on_send_as_respent(
            addr_data,
            pre,
            is_first_encounter,
            also_received,
            will_be_empty,
        );
        self.exposed
            .on_send(output_type, addr_data, pre, will_be_empty);
    }
}

impl BlockActivityCounts {
    #[inline(always)]
    pub fn active(&self) -> u32 {
        debug_assert!(self.bidirectional <= self.sending.min(self.receiving));
        self.sending + self.receiving - self.bidirectional
    }
}

impl ReuseState {
    #[inline]
    fn reset_per_block(&mut self) {
        self.output_events = 0;
        self.input_events = 0;
        self.active = 0;
    }

    #[inline]
    fn on_receive_as_reused(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        let is_now_reused = addr_data.is_reused();
        if is_now_reused && !pre.was_reused {
            self.total += 1;
            self.funded += 1;
        } else if pre.was_reused && !pre.was_funded {
            self.funded += 1;
        }

        let skip_first = 1u32.saturating_sub(pre.prev_funded_txo_count.min(1));
        self.output_events += u64::from(output_count.saturating_sub(skip_first));
        if is_now_reused {
            self.active += 1;
        }
        apply_supply_delta(
            &mut self.supply,
            pre.reused_contribution,
            addr_data.reused_supply_contribution(),
        );
    }

    #[inline]
    fn on_receive_as_respent(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        if pre.was_respent && !pre.was_funded {
            self.funded += 1;
        }
        if pre.was_respent {
            self.output_events += u64::from(output_count);
            self.active += 1;
        }
        apply_supply_delta(
            &mut self.supply,
            pre.respent_contribution,
            addr_data.respent_supply_contribution(),
        );
    }

    #[inline]
    fn on_send_as_reused(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if pre.was_reused {
            self.input_events += 1;
        }
        if is_first_encounter && pre.was_reused && !also_received {
            self.active += 1;
        }
        if will_be_empty && pre.was_reused {
            self.funded -= 1;
        }
        apply_supply_delta(
            &mut self.supply,
            pre.reused_contribution,
            addr_data.reused_supply_contribution(),
        );
    }

    #[inline]
    fn on_send_as_respent(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if pre.was_respent {
            self.input_events += 1;
        }

        let is_now_respent = addr_data.is_respent();
        if is_now_respent && !pre.was_respent {
            self.total += 1;
            if !will_be_empty {
                self.funded += 1;
            }
        }
        if (is_first_encounter && pre.was_respent && !also_received)
            || (is_now_respent && !pre.was_respent)
        {
            self.active += 1;
        }
        if will_be_empty && pre.was_respent {
            self.funded -= 1;
        }
        apply_supply_delta(
            &mut self.supply,
            pre.respent_contribution,
            addr_data.respent_supply_contribution(),
        );
    }
}

impl ExposedState {
    #[inline]
    fn on_receive(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        status: AddrReceiveStatus,
    ) {
        if !pre.was_funded && pre.was_pubkey_exposed {
            self.funded += 1;
        }
        if output_type.pubkey_exposed_at_funding() && matches!(status, AddrReceiveStatus::New) {
            self.total += 1;
        }
        apply_supply_delta(
            &mut self.supply,
            pre.exposed_contribution,
            addr_data.exposed_supply_contribution(output_type),
        );
    }

    #[inline]
    fn on_send(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        will_be_empty: bool,
    ) {
        apply_supply_delta(
            &mut self.supply,
            pre.exposed_contribution,
            addr_data.exposed_supply_contribution(output_type),
        );
        if !pre.was_pubkey_exposed {
            self.total += 1;
            if !will_be_empty {
                self.funded += 1;
            }
        }
        if will_be_empty && pre.was_pubkey_exposed {
            self.funded -= 1;
        }
    }
}

/// Apply a signed `after - before` change to an unsigned supply.
#[inline]
fn apply_supply_delta(slot: &mut Sats, before: Sats, after: Sats) {
    if after >= before {
        *slot += after - before;
    } else {
        *slot -= before - after;
    }
}

impl AddAssign for MemberState {
    fn add_assign(&mut self, rhs: Self) {
        self.funded += rhs.funded;
        self.empty += rhs.empty;
        self.supply += rhs.supply;
        self.outputs += rhs.outputs;
        self.inputs += rhs.inputs;
        self.activity += rhs.activity;
        self.reused += rhs.reused;
        self.respent += rhs.respent;
        self.exposed += rhs.exposed;
    }
}

impl AddAssign for BlockActivityCounts {
    fn add_assign(&mut self, rhs: Self) {
        self.reactivated += rhs.reactivated;
        self.sending += rhs.sending;
        self.receiving += rhs.receiving;
        self.bidirectional += rhs.bidirectional;
    }
}

impl AddAssign for ReuseState {
    fn add_assign(&mut self, rhs: Self) {
        self.funded += rhs.funded;
        self.total += rhs.total;
        self.supply += rhs.supply;
        self.output_events += rhs.output_events;
        self.input_events += rhs.input_events;
        self.active += rhs.active;
    }
}

impl AddAssign for ExposedState {
    fn add_assign(&mut self, rhs: Self) {
        self.funded += rhs.funded;
        self.total += rhs.total;
        self.supply += rhs.supply;
    }
}

impl ShardDelta for MemberState {
    #[inline]
    fn add_delta(&mut self, shard: Self, base: Self) {
        self.funded.add_delta(shard.funded, base.funded);
        self.empty.add_delta(shard.empty, base.empty);
        self.supply.add_delta(shard.supply, base.supply);
        self.outputs.add_delta(shard.outputs, base.outputs);
        self.inputs.add_delta(shard.inputs, base.inputs);
        self.activity.add_delta(shard.activity, base.activity);
        self.reused.add_delta(shard.reused, base.reused);
        self.respent.add_delta(shard.respent, base.respent);
        self.exposed.add_delta(shard.exposed, base.exposed);
    }
}

impl ShardDelta for BlockActivityCounts {
    #[inline]
    fn add_delta(&mut self, shard: Self, base: Self) {
        self.reactivated
            .add_delta(shard.reactivated, base.reactivated);
        self.sending.add_delta(shard.sending, base.sending);
        self.receiving.add_delta(shard.receiving, base.receiving);
        self.bidirectional
            .add_delta(shard.bidirectional, base.bidirectional);
    }
}

impl ShardDelta for ReuseState {
    #[inline]
    fn add_delta(&mut self, shard: Self, base: Self) {
        self.funded.add_delta(shard.funded, base.funded);
        self.total.add_delta(shard.total, base.total);
        self.supply.add_delta(shard.supply, base.supply);
        self.output_events
            .add_delta(shard.output_events, base.output_events);
        self.input_events
            .add_delta(shard.input_events, base.input_events);
        self.active.add_delta(shard.active, base.active);
    }
}

impl ShardDelta for ExposedState {
    #[inline]
    fn add_delta(&mut self, shard: Self, base: Self) {
        self.funded.add_delta(shard.funded, base.funded);
        self.total.add_delta(shard.total, base.total);
        self.supply.add_delta(shard.supply, base.supply);
    }
}
