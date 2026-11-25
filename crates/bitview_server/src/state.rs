use std::{sync::Arc, time::Instant};

#[cfg(feature = "full-api")]
use axum::body::Bytes;
#[cfg(feature = "full-api")]
use bitview_query::{PublicationCache, RepresentationId};
use bitview_query::AsyncQuery;
#[cfg(feature = "full-api")]
use brk_rpc::AsyncClient;
use derive_more::Deref;
use jiff::Timestamp;
use tokio::sync::Semaphore;

#[cfg(feature = "full-api")]
use crate::prepared_json::PreparedJson;
#[cfg(feature = "series")]
use crate::series_bodies::SeriesBodies;
use crate::CdnCacheMode;

/// One request's state: its query (carrying the request deadline) over the server's shared state.
#[derive(Clone, Deref)]
pub struct AppState {
    pub query: AsyncQuery,
    #[deref]
    shared: Arc<SharedState>,
}

/// Immutable for the server's lifetime: cloned per request as one `Arc`.
pub struct SharedState {
    /// Chain reads (tip, blocks, transactions, addresses): as many at once as there are cores.
    pub chain_query: Arc<Semaphore>,
    /// Whole-history aggregates (the mining statistics): a few at once, apart from chain reads,
    /// so a client looping on them waits behind itself, not ahead of every point read.
    #[cfg(feature = "full-api")]
    pub history_query: Arc<Semaphore>,
    /// Oracle replays (searches, histograms at a height or a day), which read every output of their
    /// blocks: one at a time, waiting apart from other reads.
    #[cfg(feature = "full-api")]
    pub oracle_replays: Arc<Semaphore>,
    #[cfg(feature = "full-api")]
    pub raw_block_bodies: Arc<Semaphore>,
    #[cfg(feature = "full-api")]
    pub historical_price_bodies: Arc<Semaphore>,
    #[cfg(feature = "full-api")]
    /// The full-history representation, shared by requests for the same publication.
    pub historical_price_cache: Arc<PublicationCache<(Bytes, RepresentationId)>>,
    #[cfg(feature = "full-api")]
    pub mempool_txid_bodies: Arc<Semaphore>,
    /// Projected blocks, under 1 MB each.
    #[cfg(feature = "full-api")]
    pub mempool_block_bodies: Arc<Semaphore>,
    /// Block template diffs: small, except right after a block.
    #[cfg(feature = "full-api")]
    pub mempool_block_diff_bodies: Arc<Semaphore>,
    #[cfg(feature = "full-api")]
    pub broadcast_requests: Arc<Semaphore>,
    /// Transaction, block and address-history bodies: taken by the large ones only, once built.
    pub chain_bodies: Arc<Semaphore>,
    /// Where broadcasts go: the only requests that ask the node.
    #[cfg(feature = "full-api")]
    pub node: AsyncClient,
    #[cfg(feature = "series")]
    pub series_bodies: Arc<SeriesBodies>,
    #[cfg(feature = "full-api")]
    pub utxo_set_query: Arc<Semaphore>,
    #[cfg(feature = "full-api")]
    pub utxo_set_bodies: Arc<Semaphore>,
    #[cfg(feature = "full-api")]
    pub urpd_query: Arc<Semaphore>,
    #[cfg(feature = "full-api")]
    pub urpd_bodies: Arc<Semaphore>,
    #[cfg(feature = "full-api")]
    pub mining_pools_body: Arc<PreparedJson>,
    pub started_at: Timestamp,
    pub started_instant: Instant,
    pub max_weight: usize,
    pub max_utxos: usize,
    pub cdn_cache_mode: CdnCacheMode,
}

impl AppState {
    pub fn new(query: AsyncQuery, shared: SharedState) -> Self {
        Self {
            query,
            shared: Arc::new(shared),
        }
    }
}
