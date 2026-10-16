use bitview_primitives::{ByAddrType, TypeIndex};
use brk_types::{Cents, Sats};
use rustc_hash::FxHashMap;

use crate::{
    addr::SHARDS,
    block::{Received, TxIndexes},
};

/// One address type's activity in one block, for one shard of its addresses. Its maps and
/// vectors allocate on first insert: most shard/type pairs of a block stay empty.
#[derive(Default)]
pub struct TypeBlock {
    pub received: FxHashMap<TypeIndex, Received>,
    /// Spends in input order, with their creation prices.
    pub spends: Vec<(TypeIndex, Sats, Cents)>,
    /// Transactions each address spent in.
    pub sent_txs: FxHashMap<TypeIndex, TxIndexes>,
}

/// One block's activity by shard and address type.
pub type ShardBlocks = [ByAddrType<TypeBlock>; SHARDS];
