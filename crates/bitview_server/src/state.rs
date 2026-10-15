use std::{future::Future, path::PathBuf, sync::Arc, time::Instant};

use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Response, header},
    response::IntoResponse,
};
#[cfg(feature = "chain")]
use bitview_query::PublicationCache;
use bitview_query::{AsyncQuery, Error as QueryError, Query, RepresentationId, Result};
use brk_rpc::AsyncClient;
#[cfg(feature = "chain")]
use brk_types::Version;
use derive_more::Deref;
use jiff::Timestamp;
use serde::Serialize;
use serde_json::to_vec;
use tokio::sync::Semaphore;

#[cfg(feature = "series")]
use crate::series_bodies::SeriesBodies;
use crate::{CacheParams, CdnCacheMode, Error, Website, extended::ResponseExtended};
#[cfg(feature = "chain")]
use crate::{prepared_json::PreparedJson, raw_body::RawBodyPermit};

/// One request's state: its query (carrying the request deadline) over the server's shared state.
#[derive(Clone, Deref)]
pub struct AppState {
    pub query: AsyncQuery,
    #[deref]
    shared: Arc<SharedState>,
}

/// Immutable for the server's lifetime: cloned per request as one `Arc`.
pub struct SharedState {
    pub sync_query: Arc<Semaphore>,
    pub disk_query: Arc<Semaphore>,
    #[cfg(feature = "chain")]
    pub raw_block_bodies: Arc<Semaphore>,
    #[cfg(feature = "chain")]
    pub historical_price_bodies: Arc<Semaphore>,
    #[cfg(feature = "chain")]
    /// The full-history representation, shared by requests for the same publication.
    pub historical_price_cache: Arc<PublicationCache<(Bytes, RepresentationId)>>,
    #[cfg(feature = "chain")]
    pub mempool_txid_bodies: Arc<Semaphore>,
    /// Block templates, about 10 MB each.
    #[cfg(feature = "chain")]
    pub block_template_bodies: Arc<Semaphore>,
    /// Block template diffs: small, except right after a block.
    #[cfg(feature = "chain")]
    pub block_template_diff_bodies: Arc<Semaphore>,
    #[cfg(feature = "chain")]
    pub broadcast_requests: Arc<Semaphore>,
    /// Transaction, block and address-history bodies.
    #[cfg(feature = "chain")]
    pub chain_bodies: Arc<Semaphore>,
    pub node: AsyncClient,
    #[cfg(feature = "series")]
    pub series_bodies: Arc<SeriesBodies>,
    #[cfg(feature = "chain")]
    pub utxo_set_query: Arc<Semaphore>,
    #[cfg(feature = "chain")]
    pub utxo_set_bodies: Arc<Semaphore>,
    #[cfg(feature = "urpd")]
    pub urpd_query: Arc<Semaphore>,
    #[cfg(feature = "urpd")]
    pub urpd_bodies: Arc<Semaphore>,
    #[cfg(feature = "chain")]
    pub mining_pools_body: Arc<PreparedJson>,
    pub data_path: PathBuf,
    pub website: Website,
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

    /// A cheap optimistic read may defer to the admitted read path, but must
    /// never turn temporary publication loss into an HTTP error.
    pub fn preflight<T>(&self, f: impl FnOnce(&Query) -> Result<Option<T>>) -> Result<Option<T>> {
        match self.query.sync(f) {
            Err(QueryError::StateUpdating) => Ok(None),
            result => result,
        }
    }

    /// [`Self::respond_admitted`] for transaction, block and address bodies.
    #[cfg(feature = "chain")]
    pub async fn respond_read<S: 'static>(
        &self,
        headers: HeaderMap,
        content_type: &'static str,
        select: impl FnOnce(&Query) -> Result<(S, CacheParams)> + Clone + Send + 'static,
        build: impl FnOnce(&Query, S) -> Result<Bytes> + Clone + Send + 'static,
    ) -> Response<Body> {
        self.respond_admitted(
            &self.sync_query,
            &self.chain_bodies,
            headers,
            content_type,
            select,
            build,
        )
        .await
        .unwrap_or_else(|error| Error::from(error).into_response())
    }

    pub async fn read_admitted<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Query) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        self.query
            .read_with_admission(Some(&self.sync_query), f)
            .await
    }

    pub async fn read_with_admission<T: Send + 'static>(
        &self,
        admission: &Arc<Semaphore>,
        f: impl FnOnce(&Query) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        self.query.read_with_admission(Some(admission), f).await
    }

    pub async fn run_with_admission<T: Send + 'static>(
        &self,
        admission: &Arc<Semaphore>,
        f: impl FnOnce(&Query) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let permit = admission
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| QueryError::Internal("query admission closed"))?;
        self.query
            .run(move |q| {
                let _permit = permit;
                f(q)
            })
            .await
    }

    pub fn assemble_response(
        params: CacheParams,
        bytes: Bytes,
        apply_content_headers: impl FnOnce(&mut HeaderMap),
    ) -> Response<Body> {
        let mut response = Response::new(Body::from(bytes));
        let headers = response.headers_mut();
        apply_content_headers(headers);
        params.apply_to(headers);
        response
    }

    /// Defer body work and content-header preparation until after revalidation.
    pub async fn respond_with_future<H: FnOnce(&mut HeaderMap)>(
        headers: &HeaderMap,
        params: CacheParams,
        body: impl Future<Output = Result<(Bytes, H)>>,
    ) -> Response<Body> {
        if params.matches_etag(headers) {
            return ResponseExtended::new_not_modified(&params);
        }

        match body.await {
            Ok((bytes, apply_content_headers)) => {
                Self::assemble_response(params, bytes, apply_content_headers)
            }
            Err(error) => Error::from(error).into_response(),
        }
    }

    fn respond_immediate(
        &self,
        headers: &HeaderMap,
        params: CacheParams,
        content_type: &'static str,
        bytes: impl FnOnce() -> Bytes,
    ) -> Response<Body> {
        if params.matches_etag(headers) {
            return ResponseExtended::new_not_modified(&params);
        }

        Self::assemble_response(params, bytes(), |headers| {
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
        })
    }

    /// Immediate JSON response whose value is only built after ETag validation.
    pub fn respond_json_immediate<T: Serialize>(
        &self,
        headers: &HeaderMap,
        params: CacheParams,
        value: impl FnOnce() -> T,
    ) -> Response<Body> {
        self.respond_immediate(headers, params, "application/json", || {
            Bytes::from(to_vec(&value()).unwrap())
        })
    }

    /// Immediate JSON response for values already resolved during preflight.
    pub fn respond_json_value<T: Serialize>(
        &self,
        headers: &HeaderMap,
        params: CacheParams,
        value: T,
    ) -> Response<Body> {
        self.respond_json_immediate(headers, params, || value)
    }

    /// Immediate text response for values already resolved during preflight.
    #[cfg(feature = "chain")]
    pub fn respond_text_value(
        &self,
        headers: &HeaderMap,
        params: CacheParams,
        value: String,
    ) -> Response<Body> {
        self.respond_immediate(headers, params, "text/plain", || Bytes::from(value))
    }

    /// Immediate JSON response whose validator is derived from its exact bytes.
    #[cfg(feature = "chain")]
    pub fn respond_json_content_value<T: Serialize>(
        &self,
        headers: &HeaderMap,
        value: T,
    ) -> Response<Body> {
        let bytes = Bytes::from(to_vec(&value).unwrap());
        self.respond_json_content_bytes(headers, bytes)
    }

    /// JSON response whose representation identity is produced with its bytes.
    #[cfg(feature = "chain")]
    pub async fn respond_json_bound(
        &self,
        headers: HeaderMap,
        version: Version,
        f: impl FnOnce(&Query) -> Result<(Vec<u8>, RepresentationId)> + Clone + Send + 'static,
    ) -> Response<Body> {
        self.respond_json_built(headers, move |q| {
            let (bytes, identity) = f(q)?;
            Ok((
                Bytes::from(bytes),
                CacheParams::representation(version, identity),
            ))
        })
        .await
    }

    /// [`Self::respond_json_content`] for values that can reach megabytes.
    #[cfg(feature = "chain")]
    pub async fn respond_json_content_admitted<T: Serialize>(
        &self,
        headers: HeaderMap,
        f: impl FnOnce(&Query) -> Result<T> + Clone + Send + 'static,
    ) -> Response<Body> {
        self.respond_json_built(headers, move |q| {
            let bytes = Bytes::from(to_vec(&f(q)?)?);
            let params = CacheParams::live_hash(RepresentationId::content_hash(&bytes));
            Ok((bytes, params))
        })
        .await
    }

    /// JSON whose validator comes with its bytes. Building is bounded by query admission; a
    /// large body then takes a `chain_bodies` permit that it keeps until sent, building again
    /// once one is free.
    #[cfg(feature = "chain")]
    async fn respond_json_built(
        &self,
        headers: HeaderMap,
        f: impl FnOnce(&Query) -> Result<(Bytes, CacheParams)> + Clone + Send + 'static,
    ) -> Response<Body> {
        fn json(headers: &mut HeaderMap) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }
        let budget = self.chain_bodies.clone();
        self.read_body(&self.sync_query, &self.chain_bodies, move |q, permit| {
            let (bytes, params) = f(q)?;
            if params.matches_etag(&headers) {
                return Ok(Some(ResponseExtended::new_not_modified(&params)));
            }
            if !RawBodyPermit::retains(bytes.len()) {
                return Ok(Some(Self::assemble_response(params, bytes, json)));
            }
            let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&budget)) else {
                return Ok(None);
            };
            Ok(Some(permit.response(params, bytes, json)))
        })
        .await
        .unwrap_or_else(|error| Error::from(error).into_response())
    }

    /// JSON response whose validator is derived from the exact serialized value.
    #[cfg(any(
        feature = "chain",
        feature = "price",
        feature = "urpd",
        all(test, feature = "series")
    ))]
    pub async fn respond_json_content<T, F>(&self, headers: &HeaderMap, f: F) -> Response<Body>
    where
        T: Serialize + Send + 'static,
        F: FnOnce(&Query) -> Result<T> + Send + 'static,
    {
        let bytes = self
            .read_admitted(move |query| {
                let value = f(query)?;
                Ok(Bytes::from(to_vec(&value)?))
            })
            .await;
        match bytes {
            Ok(bytes) => self.respond_json_content_bytes(headers, bytes),
            Err(error) => Error::from(error).into_response(),
        }
    }

    pub fn respond_json_content_bytes(&self, headers: &HeaderMap, bytes: Bytes) -> Response<Body> {
        let hash = RepresentationId::content_hash(&bytes);
        self.respond_immediate(
            headers,
            CacheParams::live_hash(hash),
            "application/json",
            || bytes,
        )
    }
}
