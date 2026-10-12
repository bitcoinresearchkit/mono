use brk_types::{AddressMempoolStats, Txid};
use rustc_hash::FxHashSet;

/// Per-address mempool record: rolling stats plus the set of live
/// txids that touch the address (used to maintain `tx_count`).
#[derive(Clone, Default)]
pub struct AddrEntry {
    pub stats: AddressMempoolStats,
    pub txids: FxHashSet<Txid>,
}
