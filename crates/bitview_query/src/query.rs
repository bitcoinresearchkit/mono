use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

#[cfg(feature = "full-api")]
use arc_swap::ArcSwap;
#[cfg(feature = "series")]
use arc_swap::ArcSwapOption;
use bitview_plugin_indexer::Indexer;
#[cfg(feature = "full-api")]
use bitview_types::OraclePayments;
use bitview_types::SyncStatus;
use brk_mempool::ReadOnlyMempool;
#[cfg(feature = "full-api")]
use brk_mempool::ReadOnlyState;
use brk_reader::Reader;
use brk_rpc::Client;
use brk_types::{BlockHash, Height, TxOut, Txid, Vout};
use rapidhash::RapidHashMap;
use vecdb::{ReadOnlyClone, ReadableVec, Ro};

use crate::{Error, OptionData, QueryPluginSet, Result, plugins::QueryPlugins};
#[cfg(feature = "full-api")]
use crate::{
    PublicationCache, ReadPin,
    api::{ChainTips, StaleBlocks, oracle::OracleChunks},
};
#[cfg(feature = "series")]
use crate::{SeriesRegistry, pin::PublishedBounds};

/// Read-only queries whose resolved chain views pin the published prefix.
/// Bare lengths and unguarded internal helpers cannot authorize chain reads.
#[derive(Clone)]
pub struct Query(pub(crate) Arc<QueryInner<'static>>, pub(crate) Option<Instant>);
pub(crate) struct QueryInner<'a> {
    #[cfg(feature = "series")]
    registry: &'a SeriesRegistry<'a>,
    plugins: QueryPlugins<'a>,
    #[cfg(feature = "full-api")]
    mempool: Option<ReadOnlyMempool>,
    #[cfg(feature = "full-api")]
    pub(crate) live_oracle: PublicationCache<brk_oracle::Oracle>,
    #[cfg(feature = "full-api")]
    pub(crate) payment_chunks: parking_lot::Mutex<OracleChunks<OraclePayments>>,
    #[cfg(feature = "full-api")]
    pub(crate) stale_blocks: ArcSwap<StaleBlocks>,
    #[cfg(feature = "full-api")]
    pub(crate) chain_tips: ChainTips,
    /// Read bounds cached for the newest publication a pin asked for.
    #[cfg(feature = "series")]
    pub(crate) bounds: ArcSwapOption<PublishedBounds>,
}

impl Query {
    /// How long a read may wait for admission and run.
    pub(crate) const READ_TIMEOUT: Duration = Duration::from_secs(4);

    /// A cheap request-local view; shared data is unchanged.
    pub(crate) fn with_deadline(&self, deadline: Instant) -> Self {
        Self(Arc::clone(&self.0), Some(deadline))
    }

    pub(crate) fn check_deadline(&self) -> Result<()> {
        if self.1.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(Error::ReadTimeout);
        }
        Ok(())
    }

    /// Builds the process-lifetime read-only query view.
    ///
    /// The cloned composition is intentionally leaked so query views can borrow
    /// it for the process lifetime. The series API also builds a catalog that
    /// borrows this composition. A daemon should call this once; repeated or
    /// multi-instance construction is outside this API's lifecycle contract.
    pub(crate) fn build<P>(plugins: &P, _mempool: Option<ReadOnlyMempool>) -> Self
    where
        P: ReadOnlyClone,
        P::ReadOnly: QueryPluginSet + 'static,
    {
        let plugin_set = Box::leak(Box::new(plugins.read_only_clone()));
        #[cfg(feature = "series")]
        let registry = Box::leak(Box::new(SeriesRegistry::build(plugin_set)));
        let plugins = QueryPlugins::new(plugin_set);

        Self(
            Arc::new(QueryInner {
                #[cfg(feature = "series")]
                registry,
                plugins,
                #[cfg(feature = "full-api")]
                mempool: _mempool,
                #[cfg(feature = "full-api")]
                live_oracle: Default::default(),
                #[cfg(feature = "full-api")]
                payment_chunks: Default::default(),
                #[cfg(feature = "full-api")]
                stale_blocks: Default::default(),
                #[cfg(feature = "full-api")]
                chain_tips: Default::default(),
                #[cfg(feature = "series")]
                bounds: ArcSwapOption::empty(),
            }),
            None,
        )
    }

    /// Resolves the confirmed outputs mempool transactions spend, as published when called.
    pub fn indexer_prevout_resolver(
        &self,
    ) -> impl Fn(&[(Txid, Vout)]) -> RapidHashMap<(Txid, Vout), TxOut> + Send + Sync + use<> {
        let query = self.clone();
        // The reads check their own bounds: the pin alone, no calendar read bounds.
        move |holes| {
            let pin = query.indexer().pin();
            query.indexer().prevouts(holes, pin.lengths())
        }
    }

    /// Tip block hash at the pipeline-safe ceiling.
    #[inline]
    pub fn tip_blockhash(&self) -> BlockHash {
        self.indexer().tip_blockhash()
    }

    /// The published tip's hash and its publication's revision, from one pin: a revision unchanged
    /// later means nothing published in between.
    pub fn tip_publication(&self) -> (BlockHash, u64) {
        let pin = self.indexer().pin();
        (pin.tip(), pin.revision())
    }

    /// The published tip's height and hash, from one pin; `None` before genesis.
    pub fn tip_block(&self) -> Option<(Height, BlockHash)> {
        let pin = self.indexer().pin();
        pin.lengths()
            .last_height()
            .map(|height| (height, pin.tip()))
    }

    /// Build sync status entirely from one safely published local snapshot.
    pub fn local_sync_status(&self) -> Result<SyncStatus> {
        self.sync_status_from(None)
    }

    /// Build sync status with the given external tip height. Both indexed and
    /// computed heights use one safely published pipeline snapshot.
    pub fn sync_status(&self, tip_height: Height) -> Result<SyncStatus> {
        self.sync_status_from(Some(tip_height))
    }

    fn sync_status_from(&self, tip_height: Option<Height>) -> Result<SyncStatus> {
        let guard = self.pin_safe_lengths()?;
        let indexed_height = guard.lengths().last_height().ok_or(Error::StateUpdating)?;
        let tip_height = tip_height.unwrap_or(indexed_height);
        let blocks_behind = Height::from(tip_height.saturating_sub(*indexed_height));
        let last_indexed_at_unix = self
            .indexer()
            .vecs()
            .blocks
            .timestamp
            .collect_one(indexed_height)
            .data()?;
        drop(guard);

        Ok(SyncStatus {
            indexed_height,
            computed_height: indexed_height,
            tip_height,
            blocks_behind,
            last_indexed_at: last_indexed_at_unix.to_iso8601(),
            last_indexed_at_unix,
        })
    }

    #[inline]
    pub(crate) fn reader(&self) -> &Reader {
        self.indexer().reader()
    }

    #[inline]
    pub fn client(&self) -> &Client {
        self.reader().client()
    }

    #[inline]
    pub fn blocks_dir(&self) -> &Path {
        self.reader().blocks_dir()
    }

    #[inline]
    pub(crate) fn indexer(&self) -> &Indexer<Ro> {
        self.0.plugins.indexer
    }

    #[cfg(feature = "series")]
    #[inline]
    pub(crate) fn plugins(&self) -> &QueryPlugins<'static> {
        &self.0.plugins
    }

    #[cfg(feature = "full-api")]
    #[inline]
    pub(crate) fn mempool(&self) -> Option<Arc<ReadOnlyState>> {
        self.0.mempool.as_ref().map(ReadOnlyMempool::load)
    }

    /// The mempool, then a pin: the pool never follows a block before the chain publishes it, so
    /// loaded first it is at the pin's tip, behind it for the moment after blocks publish, or on
    /// the branch a reorg is replacing.
    #[cfg(feature = "full-api")]
    pub(crate) fn mempool_then_pin(&self) -> Result<(Option<Arc<ReadOnlyState>>, ReadPin)> {
        let mempool = self.mempool();
        Ok((mempool, self.pin_safe_lengths()?))
    }

    /// `mempool` at its own tip, for a lookup the chain answers first: whatever the chain holds
    /// answers before it, so the pool only adds what is still pending. Views adding the pool to
    /// the chain's numbers need it as the pin sees it instead ([`Query::with_pending`]).
    #[cfg(feature = "full-api")]
    pub(crate) fn lookup_mempool(
        mempool: Option<Arc<ReadOnlyState>>,
    ) -> Option<(Arc<ReadOnlyState>, BlockHash)> {
        mempool.map(|mempool| {
            // Before its first publication the pool has no tip, and its reads answer that.
            let tip = mempool.tip().unwrap_or_default();
            (mempool, tip)
        })
    }

    #[cfg(feature = "series")]
    #[inline]
    pub fn registry(&self) -> &'static SeriesRegistry<'static> {
        self.0.registry
    }
}
