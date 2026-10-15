#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)]

use bitview_plugin_indexer::SafeLengths;

use std::{
    ops::Deref,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

#[cfg(any(feature = "series", feature = "price"))]
use arc_swap::ArcSwapOption;

use bitview_plugin_indexer::Indexer;
#[cfg(any(feature = "series", feature = "price"))]
use bitview_primitives::{Epoch, Halving, Index, Lengths};
use bitview_types::SyncStatus;
use brk_mempool::ReadOnlyMempool;
#[cfg(any(feature = "chain", feature = "price"))]
use brk_mempool::ReadOnlyState;
use brk_reader::Reader;
use brk_rpc::Client;
use brk_types::{BlockHash, Height, TxOut, Txid, Vout};
use rustc_hash::FxHashMap;
#[cfg(any(feature = "series", feature = "price"))]
use vecdb::{ReadBounds, ReadScope};
use vecdb::{ReadOnlyClone, ReadableVec, Ro};

mod r#async;
mod error;
mod output;
#[cfg(feature = "price")]
mod publication_cache;
mod query_plugin_set;
mod query_plugins;
mod representation_id;
#[cfg(feature = "series")]
mod vecs;

mod r#impl;

pub use r#async::*;
pub(crate) use error::OptionData;
pub use error::{Error, Result, SeriesNotFound};
#[cfg(feature = "urpd")]
pub use r#impl::ResolvedUrpd;
#[cfg(feature = "price")]
pub use r#impl::price::ResolvedHistoricalPrice;
#[cfg(feature = "chain")]
pub use r#impl::{
    BlockTemplateSource, MempoolTxids, ResolvedAddrChainTxs, ResolvedAddrMempoolTxs,
    ResolvedAddrTxs, ResolvedAddrUtxos, ResolvedBlockTemplateDiff, ResolvedBlockTimestamp,
    ResolvedBlocks, ResolvedBlocksV1, ResolvedConfirmedTx, ResolvedCpfp, ResolvedPoolBlocks,
    ResolvedRawTransaction, ResolvedRbf, ResolvedTransaction, UtxoSetPoint,
};
#[cfg(feature = "series")]
pub use r#impl::{ResolvedQuery, SeriesAnchor, SeriesRead};
pub use output::*;
#[cfg(feature = "price")]
pub use publication_cache::PublicationCache;
pub use query_plugin_set::{
    QueryPluginSet, SupportsAddresses, SupportsAge, SupportsBlocks, SupportsCoinflow,
    SupportsCointime, SupportsInputs, SupportsMappings, SupportsMining, SupportsOutputs,
    SupportsPools, SupportsPrice, SupportsTransactions, SupportsUtxoSet,
};
pub use query_plugins::QueryPlugins;
pub use representation_id::RepresentationId;
#[cfg(feature = "series")]
pub use vecs::{ResolvedSeriesInfo, SeriesEntry, SharedSeries, Vecs};

/// Read-only queries whose resolved chain views pin the published prefix.
/// Bare lengths and unguarded internal helpers cannot authorize chain reads.
///
///
///
///
///
#[derive(Clone)]
pub struct Query(Arc<QueryInner<'static>>, Option<Instant>);
struct QueryInner<'a> {
    #[cfg(feature = "series")]
    vecs: &'a Vecs<'a>,
    plugins: QueryPlugins<'a>,
    #[cfg(any(feature = "chain", feature = "price"))]
    mempool: Option<ReadOnlyMempool>,
    #[cfg(feature = "price")]
    live_oracle: PublicationCache<brk_oracle::Oracle>,
    /// The read bounds of the newest publication a pin asked for: one per publication, not per read.
    #[cfg(any(feature = "series", feature = "price"))]
    bounds: ArcSwapOption<PublishedBounds>,
}

#[cfg(any(feature = "series", feature = "price"))]
struct PublishedBounds {
    revision: u64,
    bounds: ReadBounds,
}

/// A pin that bounds this thread's reads, lazy calendar and count views included, by its
/// publication until it drops, so no read under it can see past it. Not `Send`: a value carried
/// to another thread keeps [`ReadPin::into_inner`]'s plain pin.
pub(crate) struct ReadPin {
    pin: SafeLengths,
    #[cfg(feature = "series")]
    bounds: ReadBounds,
    #[cfg(any(feature = "series", feature = "price"))]
    _scope: ReadScope,
}

impl ReadPin {
    /// The bounds this pin keeps this thread's reads within.
    #[cfg(feature = "series")]
    fn bounds(&self) -> &ReadBounds {
        &self.bounds
    }

    /// The pin alone, without bounding this thread's reads.
    #[cfg(any(feature = "series", feature = "price"))]
    fn into_inner(self) -> SafeLengths {
        self.pin
    }
}

impl Deref for ReadPin {
    type Target = SafeLengths;

    fn deref(&self) -> &SafeLengths {
        &self.pin
    }
}

/// `StateUpdating` for a read targeting `index` at or past `bound` (its kind's exclusive bound in
/// `pin`) while the pin is lowered: a rollback or rebuild is rewriting the canonical rows there,
/// which are hidden, not missing. Otherwise `Ok`, and the caller treats it like any row past the
/// tip (missing, or clamped); reads wholly inside the retained prefix proceed either way.
#[cfg(any(feature = "series", feature = "price"))]
pub(crate) fn check_bound<T: PartialOrd>(pin: &SafeLengths, index: T, bound: T) -> Result<()> {
    if index >= bound && pin.is_lowered() {
        return Err(Error::StateUpdating);
    }
    Ok(())
}

impl Query {
    /// How long a read may wait for admission and run.
    const READ_TIMEOUT: Duration = Duration::from_secs(4);

    /// A cheap request-local view; shared data is unchanged.
    fn with_deadline(&self, deadline: Instant) -> Self {
        Self(Arc::clone(&self.0), Some(deadline))
    }

    fn check_deadline(&self) -> Result<()> {
        if self.1.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(Error::ReadTimeout);
        }
        Ok(())
    }

    /// Pins the published state for one read: every plugin's rows below its lengths stay put
    /// (only a rollback waits for the pin; pinning never waits), and this thread's reads stay
    /// below them while the pin lives. Fails past the deadline, and while a rebuild lowered the
    /// publication to genesis: the chain is updating, not empty.
    fn pin_safe_lengths(&self) -> Result<ReadPin> {
        self.check_deadline()?;
        let pin = self.indexer().pin();
        if pin.is_lowered() && pin.lengths().height.is_zero() {
            return Err(Error::StateUpdating);
        }
        Ok(self.read_pin(pin))
    }

    /// Bounds this thread's reads by `pin` while the returned pin lives: how a value carried from
    /// an earlier read resumes reading under its pin.
    fn read_pin(&self, pin: SafeLengths) -> ReadPin {
        #[cfg(any(feature = "series", feature = "price"))]
        let bounds = self.publication_bounds(&pin);
        ReadPin {
            #[cfg(any(feature = "series", feature = "price"))]
            _scope: bounds.enter(),
            #[cfg(feature = "series")]
            bounds,
            pin,
        }
    }

    /// The read bounds of `pin`'s publication, built once per publication.
    #[cfg(any(feature = "series", feature = "price"))]
    fn publication_bounds(&self, pin: &SafeLengths) -> ReadBounds {
        let cached = self.0.bounds.load();
        if let Some(cached) = cached.as_deref()
            && cached.revision == pin.revision()
        {
            return cached.bounds.clone();
        }
        drop(cached);
        let bounds = self.read_bounds(pin.lengths());
        let published = Arc::new(PublishedBounds {
            revision: pin.revision(),
            bounds: bounds.clone(),
        });
        // An older pin never replaces a newer publication's bounds.
        self.0.bounds.rcu(|cached| match cached {
            Some(cached) if cached.revision >= published.revision => Some(Arc::clone(cached)),
            _ => Some(Arc::clone(&published)),
        });
        bounds
    }

    /// Builds the process-lifetime read-only query view.
    ///
    /// The cloned composition is intentionally leaked so query views can borrow
    /// it for the process lifetime. The series API also builds a catalog that
    /// borrows this composition. A daemon should call this once; repeated or
    /// multi-instance construction is outside this API's lifecycle contract.
    fn build<P>(plugins: &P, _mempool: Option<ReadOnlyMempool>) -> Self
    where
        P: ReadOnlyClone,
        P::ReadOnly: QueryPluginSet + 'static,
    {
        let plugin_set = Box::leak(Box::new(plugins.read_only_clone()));
        #[cfg(feature = "series")]
        let vecs = Box::leak(Box::new(Vecs::build(plugin_set)));
        let plugins = QueryPlugins::new(plugin_set);

        Self(
            Arc::new(QueryInner {
                #[cfg(feature = "series")]
                vecs,
                plugins,
                #[cfg(any(feature = "chain", feature = "price"))]
                mempool: _mempool,
                #[cfg(feature = "price")]
                live_oracle: Default::default(),
                #[cfg(any(feature = "series", feature = "price"))]
                bounds: ArcSwapOption::empty(),
            }),
            None,
        )
    }

    /// The published tip height a pin reads at (genesis while nothing is published).
    #[cfg(feature = "chain")]
    fn tip_height(pin: &SafeLengths) -> Height {
        pin.lengths().last_height().unwrap_or_default()
    }

    #[cfg(any(feature = "series", feature = "price"))]
    fn index_read_bounds(safe: Lengths) -> ReadBounds {
        let mut bounds = ReadBounds::new();

        bounds.set(Index::Height.name(), safe.height.into());
        bounds.set(Index::TxIndex.name(), safe.tx_index.into());
        bounds.set(Index::TxInIndex.name(), safe.txin_index.into());
        bounds.set(Index::TxOutIndex.name(), safe.txout_index.into());
        bounds.set(
            Index::EmptyOutputIndex.name(),
            safe.empty_output_index.into(),
        );
        bounds.set(Index::OpReturnIndex.name(), safe.op_return_index.into());
        bounds.set(Index::P2AAddressIndex.name(), safe.p2a_addr_index.into());
        bounds.set(Index::P2MSOutputIndex.name(), safe.p2ms_output_index.into());
        bounds.set(
            Index::P2PK33AddressIndex.name(),
            safe.p2pk33_addr_index.into(),
        );
        bounds.set(
            Index::P2PK65AddressIndex.name(),
            safe.p2pk65_addr_index.into(),
        );
        bounds.set(
            Index::P2PKHAddressIndex.name(),
            safe.p2pkh_addr_index.into(),
        );
        bounds.set(Index::P2SHAddressIndex.name(), safe.p2sh_addr_index.into());
        bounds.set(Index::P2TRAddressIndex.name(), safe.p2tr_addr_index.into());
        bounds.set(
            Index::P2WPKHAddressIndex.name(),
            safe.p2wpkh_addr_index.into(),
        );
        bounds.set(
            Index::P2WSHAddressIndex.name(),
            safe.p2wsh_addr_index.into(),
        );
        bounds.set(
            Index::UnknownOutputIndex.name(),
            safe.unknown_output_index.into(),
        );

        let tip = safe.last_height();
        bounds.set(
            Index::Epoch.name(),
            tip.map(|height| usize::from(Epoch::from(height)) + 1)
                .unwrap_or(0),
        );
        bounds.set(
            Index::Halving.name(),
            tip.map(|height| usize::from(Halving::from(height)) + 1)
                .unwrap_or(0),
        );

        bounds
    }

    #[cfg(any(feature = "series", feature = "price"))]
    fn read_bounds(&self, safe: Lengths) -> ReadBounds {
        let mut bounds = Self::index_read_bounds(safe);
        let timestamp = safe.last_height().and_then(|height| {
            self.plugins()
                .mappings
                .timestamp
                .monotonic
                .collect_one(height)
        });
        for index in Index::all().into_iter().filter(Index::is_date_based) {
            let len = timestamp
                .and_then(|timestamp| index.timestamp_to_index(timestamp))
                .map(|last| last + 1)
                .unwrap_or(0);
            bounds.set(index.name(), len);
        }

        bounds
    }

    /// Resolves the confirmed outputs mempool transactions spend, as published when called.
    pub fn indexer_prevout_resolver(
        &self,
    ) -> impl Fn(&[(Txid, Vout)]) -> FxHashMap<(Txid, Vout), TxOut> + Send + Sync + use<> {
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
    fn reader(&self) -> &Reader {
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
    fn indexer(&self) -> &Indexer<Ro> {
        self.0.plugins.indexer
    }

    /// The shared read-only plugin composition backing this query view.
    #[cfg(any(feature = "chain", feature = "series", feature = "price"))]
    #[inline]
    fn plugins(&self) -> &QueryPlugins<'static> {
        &self.0.plugins
    }

    #[cfg(any(feature = "chain", feature = "price"))]
    #[inline]
    fn mempool(&self) -> Option<Arc<ReadOnlyState>> {
        self.0.mempool.as_ref().map(ReadOnlyMempool::load)
    }

    /// The mempool, then a pin: the pool never follows a block before the chain publishes it, so
    /// loaded first it is at the pin's tip, behind it for the moment after blocks publish, or on
    /// the branch a reorg is replacing.
    #[cfg(any(feature = "chain", feature = "price"))]
    fn mempool_then_pin(&self) -> Result<(Option<Arc<ReadOnlyState>>, ReadPin)> {
        let mempool = self.mempool();
        Ok((mempool, self.pin_safe_lengths()?))
    }

    /// `mempool` at its own tip, for a lookup the chain answers first: whatever the chain holds
    /// answers before it, so the pool only adds what is still pending. Views adding the pool to
    /// the chain's numbers need it at the pin's tip instead ([`anchored`] at `pin.tip()`).
    ///
    /// [`anchored`]: crate::r#impl::mempool::anchored
    #[cfg(any(feature = "chain", feature = "price"))]
    fn lookup_mempool(
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
    pub fn vecs(&self) -> &'static Vecs<'static> {
        self.0.vecs
    }
}
