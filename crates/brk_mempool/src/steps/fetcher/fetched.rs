use bitcoin::Transaction as BitcoinTransaction;
use brk_rpc::MempoolState;
use brk_types::{MempoolEntryInfo, Txid};
use rapidhash::{RapidHashMap, RapidHashSet};

pub struct Fetched {
    /// Passthrough fields from the batched RPC fetch: live txid set,
    /// fee floor, chain tip. `live_txids` is the union of
    /// `getrawmempool` and `getblocktemplate` (see [`super::fetch`]),
    /// retained to build the template even when the two observations disagree.
    pub state: MempoolState,
    /// `MempoolEntryInfo` for newly-observed txids only (existing ones
    /// keep their first-sight entry on the live store).
    pub new_entries: Vec<MempoolEntryInfo>,
    pub new_txs: RapidHashMap<Txid, BitcoinTransaction>,
    /// Block 0 ordering from `getblocktemplate`. Bodies and stats have
    /// already been folded into `new_entries`/`new_txs` (or were already
    /// in the pool). The Rebuilder only needs the txid sequence to
    /// project Core's exact selection; with none (Core refused one), it packs block 0 itself.
    pub block_template_txids: Option<Vec<Txid>>,
    /// Template txids absent from the raw listing. Non-empty, the union is
    /// useful for projection, not for address balances/pages; such stale
    /// transactions never take an outpoint from a listed spender.
    pub template_only: RapidHashSet<Txid>,
    /// The new txids filled the per-cycle cap: more are left for the next cycles.
    pub capped: bool,
}
