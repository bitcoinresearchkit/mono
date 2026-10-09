use bitview_primitives::TypeIndex;

/// Addresses of each type are split by index into this many shards, processed apart.
pub const SHARDS: usize = 8;

/// An address's shard.
#[inline(always)]
pub fn shard_of(type_index: TypeIndex) -> usize {
    usize::from(type_index) % SHARDS
}
