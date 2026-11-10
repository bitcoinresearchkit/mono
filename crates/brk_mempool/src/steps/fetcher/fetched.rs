use bitcoin::Transaction as BitcoinTransaction;
use brk_rpc::MempoolState;
use brk_types::{MempoolEntryInfo, Txid};
use rapidhash::{RapidHashMap, RapidHashSet};

use crate::stores::tx_store::TxRecord;

pub struct Fetched {
    /// Passthrough fields from the batched RPC fetch: the listing (`getrawmempool`), fee floor,
    /// chain tip.
    pub state: MempoolState,
    /// Authoritative signed mining amounts; distinct from the public RPC delta list.
    pub modified_fees: Box<[(Txid, i64)]>,
    /// How many of the newest `State::unverified` records this cycle attempted.
    pub checked: usize,
    /// Attempted records whose held witness was not verified by an entry observation.
    /// Accepted replacement bodies clear these in the driver; the rest stay queued if listed.
    pub recheck: RapidHashSet<Txid>,
    /// `MempoolEntryInfo` for newly-observed txids only (existing ones
    /// keep their first-sight entry on the live store).
    pub new_entries: Vec<MempoolEntryInfo>,
    pub new_txs: RapidHashMap<Txid, BitcoinTransaction>,
    /// Block 0 ordering from `getblocktemplate`. Bodies and stats have
    /// already been folded into `new_entries`/`new_txs` (or were already
    /// in the pool). The Rebuilder only needs the txid sequence to
    /// project Core's exact selection; with none (Core refused one), it packs block 0 itself.
    pub block_template_txids: Option<Vec<Txid>>,
    /// The template's transactions the listing lacks, in its order: Core builds a template
    /// at most every few seconds while its pool changes, so these left the pool since. Their
    /// bodies serve block 0 alone; the pool buries them as it would any leaver.
    pub template_only: Vec<TxRecord>,
    /// The new txids filled the per-cycle cap: more are left for the next cycles.
    pub capped: bool,
    /// The listed txids asked for this cycle (the template's aside): one still not loaded after
    /// failed this time.
    pub requested: Vec<Txid>,
}
