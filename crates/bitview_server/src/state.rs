use std::{path::PathBuf, time::Instant};

#[cfg(feature = "chain")]
use std::{hash::Hash, sync::Arc};

use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Response, header},
    response::IntoResponse,
};
#[cfg(feature = "chain")]
use bitview_query::{
    AddrMempoolTxsPreflight, AddrTxsPreflight, BlockTemplateDiffPreflight, ResolvedAddrChainTxs,
    ResolvedAddrUtxos, ResolvedBlock, ResolvedConfirmedTx, ResolvedCpfp, ResolvedPoolBlocks,
    ResolvedRawTransaction, ResolvedRbf, ResolvedTransaction,
};
use bitview_query::{AsyncQuery, RepresentationId};
use brk_error::{Error as BrkError, Result};
#[cfg(feature = "chain")]
use brk_types::{
    Addr, AddrStats, BlockHash, MempoolBlock, NextBlockHash, PoolSlug, RecommendedFees, TimePeriod,
    TxIndex, TxStatus, Txid, TxidPrefix,
};
use brk_types::{BlockHashPrefix, Height, Version};
#[cfg(feature = "mappings")]
use brk_types::{Date, Day1};
use derive_more::Deref;
use jiff::Timestamp;
use serde::Serialize;

#[cfg(feature = "chain")]
use crate::cache::TipJsonCache;
#[cfg(feature = "series")]
use crate::series_bodies::SeriesBodies;
use crate::{CacheParams, CacheStrategy, Error, Website, extended::ResponseExtended};

#[derive(Clone, Deref)]
pub struct AppState {
    #[deref]
    pub query: AsyncQuery,
    #[cfg(feature = "series")]
    pub(crate) series_bodies: SeriesBodies,
    #[cfg(feature = "urpd")]
    pub(crate) urpd_cohorts_body: Bytes,
    #[cfg(feature = "chain")]
    pub(crate) mining_pools_body: Bytes,
    #[cfg(feature = "chain")]
    pub(crate) mining_block_fees_cache: TipJsonCache<TimePeriod>,
    pub data_path: PathBuf,
    pub website: Website,
    pub started_at: Timestamp,
    pub started_instant: Instant,
    pub max_weight: usize,
    pub max_utxos: usize,
}

impl AppState {
    pub fn tip_strategy(&self) -> CacheStrategy {
        self.sync(|q| CacheStrategy::Tip(q.tip_hash_prefix()))
    }

    /// `Immutable` if height is >6 deep, `Tip` otherwise.
    pub fn height_strategy(&self, version: Version, height: Height) -> CacheStrategy {
        self.sync(|q| {
            if height.is_deeply_confirmed(q.height()) {
                CacheStrategy::Immutable(version)
            } else {
                CacheStrategy::Tip(q.tip_hash_prefix())
            }
        })
    }

    /// `Immutable` once the following day's first height is beyond the
    /// supported reorg window, `Tip` otherwise.
    #[cfg(feature = "mappings")]
    pub async fn date_strategy(&self, version: Version, date: Date) -> Result<CacheStrategy> {
        self.run(move |query| {
            let day = Day1::try_from(date)?;
            if query.day_is_deeply_confirmed(day)? {
                Ok(CacheStrategy::Immutable(version))
            } else {
                Ok(CacheStrategy::Tip(query.tip_hash_prefix()))
            }
        })
        .await
    }

    /// Resolve complete address stats immediately when the distribution
    /// snapshot is available, including any lookup error.
    #[cfg(feature = "chain")]
    pub(crate) fn addr_stats_preflight(&self, addr: &Addr) -> Result<Option<AddrStats>> {
        self.sync(|q| q.addr_stats_preflight(addr))
    }

    /// Resolve one confirmed address page and its exact chain anchor once.
    #[cfg(feature = "chain")]
    pub(crate) fn addr_chain_txs_preflight(
        &self,
        version: Version,
        addr: &Addr,
        after_txid: Option<Txid>,
        limit: usize,
    ) -> Result<(ResolvedAddrChainTxs, CacheStrategy)> {
        self.sync(|q| {
            let resolved = q.resolve_addr_chain_txs(addr, after_txid, limit)?;
            let strategy = Self::addr_chain_txs_strategy(version, &resolved);
            Ok((resolved, strategy))
        })
    }

    #[cfg(feature = "chain")]
    pub(crate) fn addr_chain_txs_strategy(
        version: Version,
        resolved: &ResolvedAddrChainTxs,
    ) -> CacheStrategy {
        CacheStrategy::ActivityBound(version, BlockHashPrefix::from(&resolved.activity_anchor()))
    }

    /// Resolve an exact cached, chain-only, or cold combined address page.
    #[cfg(feature = "chain")]
    pub(crate) fn addr_txs_preflight(
        &self,
        addr: &Addr,
        mempool_limit: usize,
        chain_floor: usize,
        total_target: usize,
    ) -> Result<AddrTxsPreflight> {
        self.sync(|q| q.addr_txs_json_preflight(addr, mempool_limit, chain_floor, total_target))
    }

    /// Resolve an exact cached, empty, or cold address-mempool response.
    #[cfg(feature = "chain")]
    pub(crate) fn addr_mempool_txs_preflight(
        &self,
        addr: &Addr,
        limit: usize,
    ) -> Result<AddrMempoolTxsPreflight> {
        self.sync(|q| q.addr_mempool_txs_json_preflight(addr, limit))
    }

    /// Resolve a confirmed-only address UTXO query and its chain anchor once.
    #[cfg(feature = "chain")]
    pub(crate) fn addr_utxos_preflight(
        &self,
        version: Version,
        addr: &Addr,
    ) -> Result<Option<(ResolvedAddrUtxos, CacheStrategy)>> {
        self.sync(|q| {
            let Some(resolved) = q.addr_utxos_preflight(addr)? else {
                return Ok(None);
            };
            let strategy = Self::addr_utxos_strategy(version, resolved.block_hash());
            Ok(Some((resolved, strategy)))
        })
    }

    #[cfg(feature = "chain")]
    pub(crate) fn addr_utxos_strategy(version: Version, block_hash: BlockHash) -> CacheStrategy {
        CacheStrategy::ActivityBound(version, BlockHashPrefix::from(&block_hash))
    }

    /// `Immutable` if the block is >6 deep (status stable), `Tip` otherwise.
    /// For block status which changes when the next block arrives.
    #[cfg(feature = "chain")]
    pub(crate) fn block_status_preflight(
        &self,
        version: Version,
        hash: &BlockHash,
    ) -> Result<(ResolvedBlock, CacheStrategy)> {
        self.sync(|q| {
            let block = q.resolve_block(hash)?;
            let strategy = if block.height().is_deeply_confirmed(q.height()) {
                CacheStrategy::Immutable(version)
            } else {
                CacheStrategy::Tip(q.tip_hash_prefix())
            };
            Ok((block, strategy))
        })
    }

    /// Resolve an exact best-chain block once and bind its response to that hash.
    #[cfg(feature = "chain")]
    pub(crate) fn block_preflight(
        &self,
        version: Version,
        hash: &BlockHash,
    ) -> Result<(ResolvedBlock, CacheStrategy)> {
        self.sync(|q| {
            let block = q.resolve_block(hash)?;
            let strategy = CacheStrategy::BlockBound(version, BlockHashPrefix::from(hash));
            Ok((block, strategy))
        })
    }

    /// Resolve an exact confirmed transaction once and bind its response to
    /// the block that currently contains it.
    #[cfg(feature = "chain")]
    pub(crate) fn confirmed_tx_preflight(
        &self,
        version: Version,
        txid: &Txid,
    ) -> Result<(ResolvedConfirmedTx, CacheStrategy)> {
        self.sync(|q| {
            let tx = q.resolve_confirmed_tx(txid)?;
            let strategy = if tx.is_deeply_confirmed(q.height()) {
                CacheStrategy::BlockBound(version, BlockHashPrefix::from(&tx.block_hash()))
            } else {
                CacheStrategy::Tip(q.tip_hash_prefix())
            };
            Ok((tx, strategy))
        })
    }

    /// Resolve exact raw transaction bytes and derive their witness-aware cache strategy.
    #[cfg(feature = "chain")]
    pub(crate) fn raw_transaction_preflight(
        &self,
        version: Version,
        txid: &Txid,
    ) -> Result<(ResolvedRawTransaction, CacheStrategy)> {
        self.sync(|q| {
            let transaction = q.resolve_raw_transaction(txid)?;
            let strategy = Self::representation_strategy(
                version,
                transaction.identity(),
                q.height(),
                q.tip_hash_prefix(),
            );
            Ok((transaction, strategy))
        })
    }

    /// Resolve exact transaction JSON and derive its content- or block-bound cache strategy.
    #[cfg(feature = "chain")]
    pub(crate) fn transaction_preflight(
        &self,
        version: Version,
        txid: &Txid,
    ) -> Result<(ResolvedTransaction, CacheStrategy)> {
        self.sync(|q| {
            let transaction = q.resolve_transaction(txid)?;
            let strategy = Self::representation_strategy(
                version,
                transaction.identity(),
                q.height(),
                q.tip_hash_prefix(),
            );
            Ok((transaction, strategy))
        })
    }

    /// Resolve exact CPFP JSON and derive its content- or block-bound cache strategy.
    #[cfg(feature = "chain")]
    pub(crate) fn cpfp_preflight(
        &self,
        version: Version,
        txid: &Txid,
    ) -> Result<(ResolvedCpfp, CacheStrategy)> {
        self.sync(|q| {
            let cpfp = q.resolve_cpfp(txid)?;
            let strategy = Self::representation_strategy(
                version,
                cpfp.identity(),
                q.height(),
                q.tip_hash_prefix(),
            );
            Ok((cpfp, strategy))
        })
    }

    /// Resolve one exact RBF tree. Empty responses get an exact strategy here.
    #[cfg(feature = "chain")]
    pub(crate) fn rbf_preflight(
        &self,
        version: Version,
        txid: &Txid,
    ) -> Result<(ResolvedRbf, Option<CacheStrategy>)> {
        self.sync(|q| {
            let rbf = q.resolve_rbf(txid)?;
            let strategy = rbf.identity().map(|identity| {
                Self::representation_strategy(version, identity, q.height(), q.tip_hash_prefix())
            });
            Ok((rbf, strategy))
        })
    }

    #[cfg(feature = "chain")]
    fn representation_strategy(
        version: Version,
        identity: RepresentationId,
        current_height: Height,
        current_tip: BlockHashPrefix,
    ) -> CacheStrategy {
        match identity {
            RepresentationId::Content(hash) => CacheStrategy::LiveHash(hash),
            RepresentationId::Block { hash, height }
                if height.is_deeply_confirmed(current_height) =>
            {
                CacheStrategy::BlockBound(version, BlockHashPrefix::from(&hash))
            }
            RepresentationId::Block { .. } => CacheStrategy::Tip(current_tip),
        }
    }

    /// Resolve a transaction status and its exact cache strategy without
    /// dispatching a second query.
    #[cfg(feature = "chain")]
    pub(crate) fn tx_status_preflight(
        &self,
        version: Version,
        txid: &Txid,
    ) -> Result<(TxStatus, CacheStrategy)> {
        self.sync(|q| {
            let status = q.transaction_status(txid)?;
            let strategy = match status.block_hash {
                Some(hash) if status.is_deeply_confirmed(q.height()) => {
                    CacheStrategy::BlockBound(version, BlockHashPrefix::from(&hash))
                }
                Some(_) => CacheStrategy::Tip(q.tip_hash_prefix()),
                None => CacheStrategy::LiveHash(*TxidPrefix::from(txid)),
            };
            Ok((status, strategy))
        })
    }

    /// Resolve a transaction index and its finality-aware cache strategy once.
    #[cfg(feature = "chain")]
    pub(crate) fn txid_by_index_preflight(
        &self,
        version: Version,
        index: TxIndex,
    ) -> Result<(Txid, CacheStrategy)> {
        self.sync(|q| {
            let initial_tip = q.tip_hash_prefix();
            let (txid, height) = q.txid_and_height_by_index(index)?;
            let current_height = q.height();
            let current_tip = q.tip_hash_prefix();
            if current_tip != initial_tip {
                return Err(BrkError::StateUpdating);
            }
            let strategy = if height.is_deeply_confirmed(current_height) {
                CacheStrategy::Immutable(version)
            } else {
                CacheStrategy::Tip(current_tip)
            };
            Ok((txid, strategy))
        })
    }

    /// Resolve transaction first-seen times and their exact response validator
    /// from one mempool snapshot.
    #[cfg(feature = "chain")]
    pub(crate) fn transaction_times_preflight(
        &self,
        txids: &[Txid],
    ) -> Result<(Vec<u64>, CacheStrategy)> {
        self.sync(|q| {
            let (times, hash) = q.transaction_times_with_hash(txids)?;
            Ok((times, CacheStrategy::LiveHash(hash)))
        })
    }

    /// Resolve one latest pool-block page and its exact activity anchor once.
    #[cfg(feature = "chain")]
    pub(crate) fn pool_blocks_preflight(
        &self,
        version: Version,
        slug: PoolSlug,
        limit: usize,
    ) -> Result<(ResolvedPoolBlocks, CacheStrategy)> {
        self.sync(|q| {
            let resolved = q.resolve_pool_blocks(slug, None, limit)?;
            let strategy = CacheStrategy::ActivityBound(
                version,
                BlockHashPrefix::from(&resolved.activity_anchor()),
            );
            Ok((resolved, strategy))
        })
    }

    /// Resolve every projected mempool-block statistic from one snapshot.
    #[cfg(feature = "chain")]
    pub(crate) fn mempool_blocks(&self) -> Result<Vec<MempoolBlock>> {
        self.sync(|query| query.mempool_blocks())
    }

    /// Resolve recommended fees from one projected-mempool snapshot.
    #[cfg(feature = "chain")]
    pub(crate) fn recommended_fees(&self) -> Result<RecommendedFees> {
        self.sync(|query| query.recommended_fees())
    }

    /// Resolve the projected-next-block hash and its matching cache strategy once.
    #[cfg(feature = "chain")]
    pub(crate) fn mempool_hash_preflight(&self) -> Result<(NextBlockHash, CacheStrategy)> {
        self.sync(|q| {
            let hash = q.mempool_hash()?;
            let strategy = CacheStrategy::LiveHash(hash.into());
            Ok((hash, strategy))
        })
    }

    /// Resolve the order-sensitive mempool-txid validator without copying the list.
    #[cfg(feature = "chain")]
    pub(crate) fn mempool_txids_strategy(&self) -> Result<CacheStrategy> {
        self.sync(|q| q.mempool_txids_hash().map(CacheStrategy::LiveHash))
    }

    /// Return the current block-template representation when its cache is ready.
    #[cfg(feature = "chain")]
    pub(crate) fn block_template_preflight(&self) -> Result<Option<(Arc<[u8]>, RepresentationId)>> {
        self.sync(|q| q.block_template_json_cached())
    }

    /// Resolve a cached block-template diff or its historical input before ETag handling.
    #[cfg(feature = "chain")]
    pub(crate) fn block_template_diff_preflight(
        &self,
        since: NextBlockHash,
    ) -> Result<BlockTemplateDiffPreflight> {
        self.sync(|q| q.block_template_diff_json_preflight(since))
    }

    fn assemble_response(
        params: CacheParams,
        result: Result<Bytes>,
        apply_content_headers: impl FnOnce(&mut HeaderMap),
    ) -> Response<Body> {
        match result {
            Ok(bytes) => {
                let mut response = Response::new(Body::from(bytes));
                let headers = response.headers_mut();
                apply_content_headers(headers);
                params.apply_to(headers);
                response
            }
            Err(error) => Error::from(error).into_response(),
        }
    }

    /// Shared response pipeline: ETag short-circuit, body computation on the
    /// query thread, and header assembly. Used by [`AppState::respond`]
    /// (strategy-driven) and series endpoints, which build [`CacheParams`]
    /// directly from query resolution.
    pub async fn respond_with_params<F>(
        &self,
        headers: &HeaderMap,
        params: CacheParams,
        apply_content_headers: impl FnOnce(&mut HeaderMap),
        f: F,
    ) -> Response<Body>
    where
        F: FnOnce(&bitview_query::Query) -> Result<Bytes> + Send + 'static,
    {
        if params.matches_etag(headers) {
            return ResponseExtended::new_not_modified(&params);
        }

        Self::assemble_response(params, self.run(f).await, apply_content_headers)
    }

    /// Strategy-driven cached response.
    async fn respond<F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        content_type: &'static str,
        f: F,
    ) -> Response<Body>
    where
        F: FnOnce(&bitview_query::Query) -> Result<Bytes> + Send + 'static,
    {
        let expected_tip = strategy.tip_hash();
        let params = CacheParams::resolve(&strategy);
        self.respond_with_params(
            headers,
            params,
            |h| {
                h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
            },
            move |query| {
                if expected_tip.is_some_and(|tip| query.tip_hash_prefix() != tip) {
                    return Err(BrkError::StateUpdating);
                }
                let bytes = f(query)?;
                if expected_tip.is_some_and(|tip| query.tip_hash_prefix() != tip) {
                    return Err(BrkError::StateUpdating);
                }
                Ok(bytes)
            },
        )
        .await
    }

    fn respond_immediate(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        content_type: &'static str,
        bytes: impl FnOnce() -> Bytes,
    ) -> Response<Body> {
        let params = CacheParams::resolve(&strategy);
        if params.matches_etag(headers) {
            return ResponseExtended::new_not_modified(&params);
        }

        Self::assemble_response(params, Ok(bytes()), |headers| {
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
        })
    }

    /// Immediate JSON response whose value is only built after ETag validation.
    pub(crate) fn respond_json_immediate<T: Serialize>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        value: impl FnOnce() -> T,
    ) -> Response<Body> {
        self.respond_immediate(headers, strategy, "application/json", || {
            Bytes::from(serde_json::to_vec(&value()).unwrap())
        })
    }

    /// Immediate JSON response for values already resolved during preflight.
    pub fn respond_json_value<T: Serialize>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        value: T,
    ) -> Response<Body> {
        self.respond_json_immediate(headers, strategy, || value)
    }

    /// Immediate response for JSON bytes serialized once by the server.
    #[cfg(any(feature = "chain", feature = "series", feature = "urpd"))]
    pub(crate) fn respond_json_bytes_value(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        bytes: &Bytes,
    ) -> Response<Body> {
        self.respond_immediate(headers, strategy, "application/json", || bytes.clone())
    }

    /// Immediate text response for values already resolved during preflight.
    #[cfg(feature = "chain")]
    pub(crate) fn respond_text_value(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        value: String,
    ) -> Response<Body> {
        self.respond_immediate(headers, strategy, "text/plain", || Bytes::from(value))
    }

    /// Immediate JSON response whose validator is derived from its exact bytes.
    #[cfg(feature = "chain")]
    pub(crate) fn respond_json_content_value<T: Serialize>(
        &self,
        headers: &HeaderMap,
        value: T,
    ) -> Response<Body> {
        let bytes = Bytes::from(serde_json::to_vec(&value).unwrap());
        self.respond_json_content_bytes(headers, bytes)
    }

    /// JSON response with HTTP cache validation.
    pub async fn respond_json<T, F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        f: F,
    ) -> Response<Body>
    where
        T: Serialize + Send + 'static,
        F: FnOnce(&bitview_query::Query) -> Result<T> + Send + 'static,
    {
        self.respond(headers, strategy, "application/json", move |q| {
            let value = f(q)?;
            Ok(Bytes::from(serde_json::to_vec(&value).unwrap()))
        })
        .await
    }

    /// Serve one serialized JSON representation per key and exact chain tip.
    #[cfg(feature = "chain")]
    pub(crate) async fn respond_json_tip_cached<K, T, F>(
        &self,
        headers: &HeaderMap,
        cache: &TipJsonCache<K>,
        key: K,
        f: F,
    ) -> Response<Body>
    where
        K: Clone + Eq + Hash,
        T: Serialize + Send + 'static,
        F: FnOnce(&bitview_query::Query) -> Result<T> + Send + 'static,
    {
        let tip = self.sync(|query| query.tip_hash_prefix());
        let params = CacheParams::resolve(&CacheStrategy::Tip(tip));
        if params.matches_etag(headers) {
            return ResponseExtended::new_not_modified(&params);
        }

        let result = cache
            .get_or_try_insert_with(key, tip, || {
                self.run(move |query| {
                    if query.tip_hash_prefix() != tip {
                        return Err(BrkError::StateUpdating);
                    }
                    let value = f(query)?;
                    let bytes = Bytes::from(serde_json::to_vec(&value).unwrap());
                    if query.tip_hash_prefix() != tip {
                        return Err(BrkError::StateUpdating);
                    }
                    Ok(bytes)
                })
            })
            .await;

        Self::assemble_response(params, result, |headers| {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        })
    }

    /// Pre-serialized JSON response with HTTP cache validation.
    pub async fn respond_json_bytes<F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        f: F,
    ) -> Response<Body>
    where
        F: FnOnce(&bitview_query::Query) -> Result<Vec<u8>> + Send + 'static,
    {
        self.respond(headers, strategy, "application/json", move |q| {
            f(q).map(Bytes::from)
        })
        .await
    }

    /// JSON response whose representation identity is produced with its bytes.
    #[cfg(feature = "chain")]
    pub async fn respond_json_bound<F>(
        &self,
        headers: &HeaderMap,
        version: Version,
        f: F,
    ) -> Response<Body>
    where
        F: FnOnce(&bitview_query::Query) -> Result<(Vec<u8>, RepresentationId)> + Send + 'static,
    {
        let request_headers = headers.clone();
        let outcome = self
            .run(move |query| {
                let initial_tip = query.tip_hash_prefix();
                let (bytes, identity) = f(query)?;
                let current_height = query.height();
                let current_tip = query.tip_hash_prefix();
                if matches!(identity, RepresentationId::Block { .. }) && initial_tip != current_tip
                {
                    return Err(BrkError::StateUpdating);
                }
                let strategy =
                    Self::representation_strategy(version, identity, current_height, current_tip);
                let params = CacheParams::resolve(&strategy);
                if params.matches_etag(&request_headers) {
                    return Ok((params, None));
                }
                Ok((params, Some(Bytes::from(bytes))))
            })
            .await;

        let (params, body) = match outcome {
            Ok((params, None)) => return ResponseExtended::new_not_modified(&params),
            Ok((params, Some(body))) => (params, body),
            Err(error) => return Error::from(error).into_response(),
        };
        Self::assemble_response(params, Ok(body), |headers| {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        })
    }

    /// JSON response whose validator is derived from the exact serialized value.
    pub(crate) async fn respond_json_content<T, F>(
        &self,
        headers: &HeaderMap,
        f: F,
    ) -> Response<Body>
    where
        T: Serialize + Send + 'static,
        F: FnOnce(&bitview_query::Query) -> Result<T> + Send + 'static,
    {
        let bytes = self
            .run(move |query| {
                let value = f(query)?;
                Ok(Bytes::from(serde_json::to_vec(&value).unwrap()))
            })
            .await;
        match bytes {
            Ok(bytes) => self.respond_json_content_bytes(headers, bytes),
            Err(error) => Error::from(error).into_response(),
        }
    }

    fn respond_json_content_bytes(&self, headers: &HeaderMap, bytes: Bytes) -> Response<Body> {
        let RepresentationId::Content(hash) = RepresentationId::content(&bytes) else {
            unreachable!("content identity constructor returned a block identity");
        };
        self.respond_immediate(
            headers,
            CacheStrategy::LiveHash(hash),
            "application/json",
            || bytes,
        )
    }

    #[cfg(feature = "chain")]
    fn respond_bound_json(
        &self,
        headers: &HeaderMap,
        version: Version,
        bytes: Bytes,
        identity: RepresentationId,
    ) -> Response<Body> {
        let (current_height, tip) = self.sync(|q| (q.height(), q.tip_hash_prefix()));
        let strategy = Self::representation_strategy(version, identity, current_height, tip);
        let params = CacheParams::resolve(&strategy);
        if params.matches_etag(headers) {
            return ResponseExtended::new_not_modified(&params);
        }
        Self::assemble_response(params, Ok(bytes), |headers| {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        })
    }

    /// Serve already-cached JSON bytes immediately without copying them.
    #[cfg(feature = "chain")]
    pub(crate) fn respond_json_cached_value(
        &self,
        headers: &HeaderMap,
        version: Version,
        bytes: Arc<[u8]>,
        identity: RepresentationId,
    ) -> Response<Body> {
        self.respond_bound_json(headers, version, Bytes::from_owner(bytes), identity)
    }

    /// Serve a lazily cached JSON representation without copying its bytes.
    #[cfg(feature = "chain")]
    pub(crate) async fn respond_json_cached<F>(
        &self,
        headers: &HeaderMap,
        version: Version,
        cached: Option<(Arc<[u8]>, RepresentationId)>,
        build: F,
    ) -> Response<Body>
    where
        F: FnOnce(&bitview_query::Query) -> Result<(Arc<[u8]>, RepresentationId)> + Send + 'static,
    {
        let resolved = match cached {
            Some(value) => Ok(value),
            None => self.run(build).await,
        };
        match resolved {
            Ok((bytes, identity)) => {
                self.respond_json_cached_value(headers, version, bytes, identity)
            }
            Err(error) => Error::from(error).into_response(),
        }
    }

    /// JSON response where the strategy depends on the loaded value.
    ///
    /// An exact preflight strategy for the requested representation can return
    /// 304 before any work is done. Otherwise the closure runs on a blocking
    /// thread and returns both the value and its actual strategy (e.g.
    /// `Immutable` if deeply confirmed, `Tip` otherwise).
    pub async fn respond_json_adaptive<T, F>(
        &self,
        headers: &HeaderMap,
        preflight: Option<CacheStrategy>,
        f: F,
    ) -> Response<Body>
    where
        T: Serialize + Send + 'static,
        F: FnOnce(&bitview_query::Query, BlockHashPrefix) -> Result<(T, CacheStrategy)>
            + Send
            + 'static,
    {
        if let Some(strategy) = preflight {
            let params = CacheParams::resolve(&strategy);
            if params.matches_etag(headers) {
                return ResponseExtended::new_not_modified(&params);
            }
        }

        let request_headers = headers.clone();
        let outcome = self
            .run(move |query| {
                let initial_tip = query.tip_hash_prefix();
                let (value, strategy) = f(query, initial_tip)?;
                if strategy
                    .tip_hash()
                    .is_some_and(|tip| tip != initial_tip || query.tip_hash_prefix() != tip)
                {
                    return Err(BrkError::StateUpdating);
                }
                let params = CacheParams::resolve(&strategy);
                if params.matches_etag(&request_headers) {
                    return Ok((params, None));
                }
                let body = Bytes::from(serde_json::to_vec(&value).unwrap());
                Ok((params, Some(body)))
            })
            .await;

        let (params, body) = match outcome {
            Ok((params, None)) => return ResponseExtended::new_not_modified(&params),
            Ok((params, Some(body))) => (params, body),
            Err(error) => return Error::from(error).into_response(),
        };
        Self::assemble_response(params, Ok(body), |headers| {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        })
    }

    /// Text response with HTTP cache validation.
    pub async fn respond_text<F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        f: F,
    ) -> Response<Body>
    where
        F: FnOnce(&bitview_query::Query) -> Result<String> + Send + 'static,
    {
        self.respond(headers, strategy, "text/plain", move |q| {
            let value = f(q)?;
            Ok(Bytes::from(value))
        })
        .await
    }

    /// Binary response with HTTP cache validation.
    pub async fn respond_bytes<T, F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        f: F,
    ) -> Response<Body>
    where
        T: Into<Vec<u8>> + Send + 'static,
        F: FnOnce(&bitview_query::Query) -> Result<T> + Send + 'static,
    {
        self.respond(headers, strategy, "application/octet-stream", move |q| {
            let value = f(q)?;
            Ok(Bytes::from(value.into()))
        })
        .await
    }
}
