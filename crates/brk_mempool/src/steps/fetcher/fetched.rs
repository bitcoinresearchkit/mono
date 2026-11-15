use bitcoin::Transaction as BitcoinTransaction;
use brk_rpc::MempoolState;
use brk_types::{MempoolEntryInfo, Txid};
use rapidhash::{RapidHashMap, RapidHashSet};

pub struct Fetched {
    /// Passthrough fields from the batched RPC fetch: the listing (`getrawmempool`), fee floor,
    /// chain tip.
    pub state: MempoolState,
    /// The node's signed modified fees of prioritised transactions (`getprioritisedtransactions`),
    /// what mining ranks them by.
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
    /// The new txids filled the per-cycle cap: more are left for the next cycles.
    pub capped: bool,
    /// The listed txids asked for this cycle: one still not loaded after failed this time.
    pub requested: Vec<Txid>,
}
