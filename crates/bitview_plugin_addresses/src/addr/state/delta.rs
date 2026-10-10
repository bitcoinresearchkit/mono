use brk_types::Sats;

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
