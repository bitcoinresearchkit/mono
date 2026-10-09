use bitview_cohort::ByAddrType;
use brk_types::{OutputType, Sats};

use crate::addr::BlockActivityCounts;

/// A value several shards updated from the same base: the total is the base plus each
/// shard's change, wrapping since one shard's change can be negative.
pub trait ShardDelta: Copy {
    fn add_delta(&mut self, shard: Self, base: Self);
}

impl ShardDelta for u64 {
    #[inline]
    fn add_delta(&mut self, shard: Self, base: Self) {
        *self = self.wrapping_add(shard.wrapping_sub(base));
    }
}

impl ShardDelta for u32 {
    #[inline]
    fn add_delta(&mut self, shard: Self, base: Self) {
        *self = self.wrapping_add(shard.wrapping_sub(base));
    }
}

impl ShardDelta for Sats {
    #[inline]
    fn add_delta(&mut self, shard: Self, base: Self) {
        let mut value = **self;
        value.add_delta(*shard, *base);
        *self = Sats::from(value);
    }
}

impl ShardDelta for BlockActivityCounts {
    #[inline]
    fn add_delta(&mut self, shard: Self, base: Self) {
        let Self {
            reactivated,
            sending,
            receiving,
            bidirectional,
        } = shard;
        self.reactivated.add_delta(reactivated, base.reactivated);
        self.sending.add_delta(sending, base.sending);
        self.receiving.add_delta(receiving, base.receiving);
        self.bidirectional
            .add_delta(bidirectional, base.bidirectional);
    }
}

/// Adds one shard's change of `output_type`'s value.
#[inline]
pub fn add_type_delta<T: ShardDelta>(
    total: &mut ByAddrType<T>,
    shard: &ByAddrType<T>,
    base: &ByAddrType<T>,
    output_type: OutputType,
) {
    total.get_mut_unwrap(output_type).add_delta(
        *shard.get_unwrap(output_type),
        *base.get_unwrap(output_type),
    );
}
