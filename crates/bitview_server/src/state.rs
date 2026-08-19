use std::{path::PathBuf, time::Instant};

use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Response, Uri, header},
};
use bitview_query::AsyncQuery;
use brk_types::{
    Addr, BlockHash, BlockHashPrefix, Date, Height, ONE_HOUR_IN_SEC, PoolSlug,
    Timestamp as BrkTimestamp, Txid, Version,
};
use derive_more::Deref;
use jiff::Timestamp;
use serde::Serialize;
use vecdb::ReadableVec;

use crate::{CacheParams, CacheStrategy, Error, Website, extended::ResponseExtended};

#[derive(Clone, Deref)]
pub struct AppState {
    #[deref]
    pub query: AsyncQuery,
    pub data_path: PathBuf,
    pub website: Website,
    pub started_at: Timestamp,
    pub started_instant: Instant,
    pub max_weight: usize,
    pub max_utxos: usize,
}

impl AppState {
    /// `Immutable` if height is >6 deep, `Tip` otherwise.
    pub fn height_strategy(&self, version: Version, height: Height) -> CacheStrategy {
        let is_deep = self.sync(|q| (*q.height()).saturating_sub(*height) > 6);
        if is_deep {
            CacheStrategy::Immutable(version)
        } else {
            CacheStrategy::Tip
        }
    }

    /// `Immutable` if timestamp is >6 hours old (block definitely >6 deep), `Tip` otherwise.
    pub fn timestamp_strategy(&self, version: Version, timestamp: BrkTimestamp) -> CacheStrategy {
        if (*BrkTimestamp::now()).saturating_sub(*timestamp) > 6 * ONE_HOUR_IN_SEC {
            CacheStrategy::Immutable(version)
        } else {
            CacheStrategy::Tip
        }
    }

    /// `Immutable` if `date` is strictly before the indexed tip's date, `Tip` otherwise.
    /// For per-date files that keep being rewritten while the tip is still within the
    /// date's day, then settle once the tip crosses the day boundary.
    pub fn date_strategy(&self, version: Version, date: Date) -> CacheStrategy {
        self.sync(|q| {
            let height = q.height();
            q.indexer()
                .vecs()
                .blocks
                .timestamp
                .collect_one(height)
                .map(|ts| {
                    if date < Date::from(ts) {
                        CacheStrategy::Immutable(version)
                    } else {
                        CacheStrategy::Tip
                    }
                })
                .unwrap_or(CacheStrategy::Tip)
        })
    }

    /// Smart address caching. Checks mempool activity first (unless `chain_only`), then on-chain.
    /// - Address has mempool txs: `MempoolHash(addr_specific_hash)`
    /// - No mempool, has on-chain activity: `BlockBound(last_activity_block)`
    /// - Unknown address: `Tip`
    ///
    /// `before_txid` narrows the on-chain branch to the newest activity strictly
    /// older than the cursor, so paginated chain pages stay cacheable when newer
    /// activity arrives above the cursor.
    pub fn addr_strategy(
        &self,
        version: Version,
        addr: &Addr,
        chain_only: bool,
        before_txid: Option<&Txid>,
    ) -> CacheStrategy {
        self.sync(|q| {
            if !chain_only && let Some(mempool_hash) = q.addr_mempool_hash(addr) {
                return CacheStrategy::MempoolHash(mempool_hash);
            }
            q.addr_last_activity_height(addr, before_txid)
                .and_then(|h| {
                    let block_hash = q.block_hash_by_height(h)?;
                    Ok(CacheStrategy::BlockBound(
                        version,
                        BlockHashPrefix::from(&block_hash),
                    ))
                })
                .unwrap_or(CacheStrategy::Tip)
        })
    }

    /// `Immutable` if the block is >6 deep (status stable), `Tip` otherwise.
    /// For block status which changes when the next block arrives.
    pub fn block_status_strategy(&self, version: Version, hash: &BlockHash) -> CacheStrategy {
        self.sync(|q| {
            q.height_by_hash(hash)
                .map(|h| {
                    if (*q.height()).saturating_sub(*h) > 6 {
                        CacheStrategy::Immutable(version)
                    } else {
                        CacheStrategy::Tip
                    }
                })
                .unwrap_or(CacheStrategy::Tip)
        })
    }

    /// `BlockBound` if the block exists (reorg-safe via block hash), `Tip` if not found.
    pub fn block_strategy(&self, version: Version, hash: &BlockHash) -> CacheStrategy {
        self.sync(|q| {
            if q.height_by_hash(hash).is_ok() {
                CacheStrategy::BlockBound(version, BlockHashPrefix::from(hash))
            } else {
                CacheStrategy::Tip
            }
        })
    }

    /// Mempool → `MempoolHash`, confirmed → `BlockBound`, unknown → `Tip`.
    pub fn tx_strategy(&self, version: Version, txid: &Txid) -> CacheStrategy {
        self.sync(|q| {
            if let Some(mempool) = q.mempool()
                && mempool.contains_txid(txid)
            {
                return CacheStrategy::MempoolHash(mempool.next_block_hash().into());
            }
            if let Ok((_, height)) = q.resolve_tx(txid)
                && let Ok(block_hash) = q.block_hash_by_height(height)
            {
                return CacheStrategy::BlockBound(version, BlockHashPrefix::from(&block_hash));
            }
            CacheStrategy::Tip
        })
    }

    /// `BlockBound` on the pool's last-mined block hash, `Tip` if the pool has
    /// never mined. Lets the no-cursor pool-blocks page stay cached when *other*
    /// pools mine; only invalidates when this pool itself mines.
    pub fn pool_blocks_strategy(&self, version: Version, slug: PoolSlug) -> CacheStrategy {
        self.sync(|q| {
            let last = q
                .computer()
                .pools
                .pool_heights
                .latest_height(slug, q.height());
            match last.and_then(|h| q.block_hash_by_height(h).ok()) {
                Some(hash) => CacheStrategy::BlockBound(version, BlockHashPrefix::from(&hash)),
                None => CacheStrategy::Tip,
            }
        })
    }

    pub fn mempool_strategy(&self) -> CacheStrategy {
        let hash = self.sync(|q| q.mempool().map(|m| m.next_block_hash().into()).unwrap_or(0));
        CacheStrategy::MempoolHash(hash)
    }

    /// Shared response pipeline: etag short-circuit, body computation on the
    /// query thread, header assembly. Used by [`AppState::respond`]
    /// (strategy-driven) and the series endpoint (which builds [`CacheParams`]
    /// directly from query resolution).
    pub async fn respond_with_params<F>(
        &self,
        headers: &HeaderMap,
        _uri: &Uri,
        params: CacheParams,
        apply_content_headers: impl FnOnce(&mut HeaderMap),
        f: F,
    ) -> Response<Body>
    where
        F: FnOnce(&bitview_query::Query) -> brk_error::Result<Bytes> + Send + 'static,
    {
        if params.matches_etag(headers) {
            return ResponseExtended::new_not_modified(&params);
        }

        match self.run(f).await {
            Ok(bytes) => {
                let mut response = Response::new(Body::from(bytes));
                let h = response.headers_mut();
                apply_content_headers(h);
                params.apply_to(h);
                response
            }
            Err(e) => crate::error::into_response_with_etag(Error::from(e), params.etag.clone()),
        }
    }

    /// Strategy-driven cached response.
    async fn respond<F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        uri: &Uri,
        content_type: &'static str,
        f: F,
    ) -> Response<Body>
    where
        F: FnOnce(&bitview_query::Query) -> brk_error::Result<Bytes> + Send + 'static,
    {
        let tip = self.sync(|q| q.tip_hash_prefix());
        let params = CacheParams::resolve(&strategy, tip);
        self.respond_with_params(
            headers,
            uri,
            params,
            |h| {
                h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
            },
            f,
        )
        .await
    }

    /// JSON response with HTTP + server-side caching
    pub async fn respond_json<T, F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        uri: &Uri,
        f: F,
    ) -> Response<Body>
    where
        T: Serialize + Send + 'static,
        F: FnOnce(&bitview_query::Query) -> brk_error::Result<T> + Send + 'static,
    {
        self.respond(headers, strategy, uri, "application/json", move |q| {
            let value = f(q)?;
            Ok(Bytes::from(serde_json::to_vec(&value).unwrap()))
        })
        .await
    }

    /// JSON response where the strategy depends on the loaded value.
    ///
    /// Clients already holding `optimistic`'s ETag get a 304 before any work
    /// is done. Otherwise the closure runs on a blocking thread and returns
    /// both the value and the actual strategy (e.g. `Immutable` if deeply
    /// confirmed, `Tip` otherwise). Errors fall back to `Tip`. Use for
    /// resources whose freshness category depends on the data itself
    /// (outspends, threshold-based block status).
    pub async fn respond_json_optimistic<T, F>(
        &self,
        headers: &HeaderMap,
        optimistic: CacheStrategy,
        uri: &Uri,
        f: F,
    ) -> Response<Body>
    where
        T: Serialize + Send + 'static,
        F: FnOnce(&bitview_query::Query) -> brk_error::Result<(T, CacheStrategy)> + Send + 'static,
    {
        let tip = self.sync(|q| q.tip_hash_prefix());
        let optimistic_params = CacheParams::resolve(&optimistic, tip);
        if optimistic_params.matches_etag(headers) {
            return ResponseExtended::new_not_modified(&optimistic_params);
        }

        let (value_result, strategy) = match self.run(f).await {
            Ok((v, s)) => (Ok(v), s),
            Err(e) => (Err(e), CacheStrategy::Tip),
        };
        let params = CacheParams::resolve(&strategy, tip);
        self.respond_with_params(
            headers,
            uri,
            params,
            |h| {
                h.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
            },
            move |_q| {
                let value = value_result?;
                Ok(Bytes::from(serde_json::to_vec(&value).unwrap()))
            },
        )
        .await
    }

    /// Text response with HTTP + server-side caching
    pub async fn respond_text<T, F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        uri: &Uri,
        f: F,
    ) -> Response<Body>
    where
        T: AsRef<str> + Send + 'static,
        F: FnOnce(&bitview_query::Query) -> brk_error::Result<T> + Send + 'static,
    {
        self.respond(headers, strategy, uri, "text/plain", move |q| {
            let value = f(q)?;
            Ok(Bytes::from(value.as_ref().as_bytes().to_vec()))
        })
        .await
    }

    /// Binary response with HTTP + server-side caching
    pub async fn respond_bytes<T, F>(
        &self,
        headers: &HeaderMap,
        strategy: CacheStrategy,
        uri: &Uri,
        f: F,
    ) -> Response<Body>
    where
        T: Into<Vec<u8>> + Send + 'static,
        F: FnOnce(&bitview_query::Query) -> brk_error::Result<T> + Send + 'static,
    {
        self.respond(
            headers,
            strategy,
            uri,
            "application/octet-stream",
            move |q| {
                let value = f(q)?;
                Ok(Bytes::from(value.into()))
            },
        )
        .await
    }
}
