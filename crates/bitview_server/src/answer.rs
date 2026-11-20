//! One way to answer a read. A route resolves an [`Answer`]: the tag naming what it would send,
//! how long its bytes stay what they are, and how to build it. The `304`, the body budget and the
//! cache class follow from it here, so no route chooses them.

use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderName, HeaderValue, header},
    response::{IntoResponse, Response},
};
use bitview_http::{IMMUTABLE, REVALIDATE};
#[cfg(feature = "full-api")]
use bitview_query::RepresentationId;
use bitview_query::{Error as QueryError, Query, Result};
#[cfg(feature = "full-api")]
use brk_types::Version;
use brk_types::tag_hash;
use serde::Serialize;
use serde_json::to_vec;
use tokio::{
    sync::Semaphore,
    time::{Instant, timeout_at},
};

use crate::{AppState, CdnCacheMode, Error, etag::Etag, gates::Gate, raw_body::RawBodyPermit};

/// What a route answers with, resolved before any of the body is built.
pub struct Answer {
    tag: Etag,
    kind: Kind,
    build: Build,
}

/// Runs on the worker that resolved the answer, so it may hold what that pinned.
type Build = Box<dyn FnOnce(&Query) -> Result<Content>>;

/// How long the bytes at an answer's URL stay what they are: all its cache class follows from.
#[derive(Clone, Copy)]
enum Kind {
    /// They change at any time: the tip, the mempool, whatever the URL doesn't pin.
    Live,
    /// Derived from chain data the URL pins below the reorg window (JSON, CSV, the hash at a
    /// height), or prepared at startup from what only a release changes (catalogs,
    /// specifications): a release may reshape them, a deeper reorg replace them.
    Settled,
    /// Committed to by the hash in the URL, whose block lies below the reorg window (a raw
    /// block, a header, a transaction's hex, its proof): no release changes a byte of them.
    Committed,
}

impl Kind {
    /// The browser's policy, then the CDN's. Browsers are never purged, so only what no release
    /// changes is immutable for them; a second's reuse otherwise, which lets readers polling
    /// together share one answer. The CDN of a server run with `--cdn` is purged by its operator
    /// on every release and on a reorg deeper than the reorg window, so it keeps every settled
    /// answer until then.
    fn policies(self, cdn_cache_mode: CdnCacheMode) -> (&'static str, &'static str) {
        match (self, cdn_cache_mode) {
            (Self::Committed, _) => (IMMUTABLE, IMMUTABLE),
            (Self::Settled, CdnCacheMode::Aggressive) => (REVALIDATE, IMMUTABLE),
            (Self::Settled, CdnCacheMode::Live) | (Self::Live, _) => (REVALIDATE, REVALIDATE),
        }
    }
}

/// Declares an answer's kind from whether what its URL pins is settled: [`Answer::settled`] or
/// [`Answer::committed`], for a helper several routes answer through.
#[cfg(feature = "full-api")]
pub type Settle = fn(Answer, bool) -> Answer;

impl Answer {
    /// An answer `tag` names: a request holding the tag gets `304`, and `build` never runs. The
    /// tag must change whenever the bytes at this URL do, within a release (every tag starts
    /// with it), so it is made of what `build` reads, read under the same pin.
    pub fn new(
        tag: impl Into<Etag>,
        build: impl FnOnce(&Query) -> Result<Content> + 'static,
    ) -> Self {
        Self {
            tag: tag.into(),
            kind: Kind::Live,
            build: Box::new(build),
        }
    }

    /// [`Self::new`] for the JSON of the value `value` reads.
    pub fn json<T: Serialize>(
        tag: impl Into<Etag>,
        value: impl FnOnce(&Query) -> Result<T> + 'static,
    ) -> Self {
        Self::new(tag, move |q| Content::json(&value(q)?))
    }

    /// An answer whose content came with its name: `tag` saves a request holding it the
    /// transfer, not the work.
    pub fn ready(tag: impl Into<Etag>, content: Content) -> Self {
        Self::new(tag, move |_| Ok(content))
    }

    /// An answer only its own bytes name: for what is as cheap to build as to name, or has no
    /// cheaper identity.
    pub fn built(content: Content) -> Self {
        Self::ready(Etag::content(tag_hash(&content.bytes)), content)
    }

    /// For an answer the server shapes: whether what its URL pins lies below the reorg window,
    /// so that only a release or a deeper reorg changes the bytes at it.
    pub fn settled(mut self, settled: bool) -> Self {
        if settled {
            self.kind = Kind::Settled;
        }
        self
    }

    /// For an answer the hash in its URL commits to, byte for byte: whether its block lies below
    /// the reorg window, so that nothing short of a deeper reorg changes the bytes at it.
    pub fn committed(mut self, settled: bool) -> Self {
        if settled {
            self.kind = Kind::Committed;
        }
        self
    }
}

pub const JSON: &str = "application/json";

/// A built body and the headers describing it.
pub struct Content {
    content_type: &'static str,
    bytes: Bytes,
    header: Option<(HeaderName, HeaderValue)>,
}

impl Content {
    pub fn new(content_type: &'static str, bytes: impl Into<Bytes>) -> Self {
        Self {
            content_type,
            bytes: bytes.into(),
            header: None,
        }
    }

    pub fn json(value: &impl Serialize) -> Result<Self> {
        Ok(Self::new(JSON, to_vec(value)?))
    }

    pub fn text(text: impl Into<Bytes>) -> Self {
        Self::new("text/plain", text)
    }

    /// One more header describing the body (a file name, a `HEAD`'s length).
    pub fn with(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.header = Some((name, value));
        self
    }
}

/// What every attempt at an answer needs of its request and of the server.
struct Asked {
    /// The request's `If-None-Match` fields alone.
    validators: HeaderMap,
    cdn_cache_mode: CdnCacheMode,
    bodies: Option<Arc<Semaphore>>,
    reserve: bool,
}

impl Asked {
    fn new(state: &AppState, headers: &HeaderMap, gate: &Gate) -> Self {
        let mut validators = HeaderMap::new();
        for value in headers.get_all(header::IF_NONE_MATCH) {
            validators.append(header::IF_NONE_MATCH, value.clone());
        }
        Self {
            validators,
            cdn_cache_mode: state.cdn_cache_mode,
            bodies: gate.bodies.cloned(),
            reserve: gate.reserve,
        }
    }

    /// The response `answer` makes: `304` for a request holding its tag, else its body, both
    /// under the answer's cache class. `None` when the body needs a permit and none is free: a
    /// reserved one before it is built, any other once built and found large.
    fn respond(
        &self,
        q: &Query,
        permit: Option<RawBodyPermit>,
        answer: Answer,
    ) -> Result<Option<Response>> {
        let Answer { tag, kind, build } = answer;
        let mut response = if tag.matches(&self.validators) {
            bitview_http::not_modified()
        } else {
            let take = || self.bodies.as_ref().map(RawBodyPermit::try_acquire);
            let permit = match permit {
                None if self.reserve => take().flatten(),
                permit => permit,
            };
            if self.reserve && permit.is_none() {
                return Ok(None);
            }
            let Content {
                content_type,
                bytes,
                header,
            } = build(q)?;
            let bytes = if RawBodyPermit::retains(bytes.len()) {
                match permit.map(Some).or_else(take) {
                    // Kept until the last byte is sent.
                    Some(Some(permit)) => permit.bytes(bytes),
                    Some(None) => return Ok(None),
                    // No budget bounds this route's bodies.
                    None => bytes,
                }
            } else {
                bytes
            };
            let mut response = Response::new(Body::from(bytes));
            let headers = response.headers_mut();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
            if let Some((name, value)) = header {
                headers.insert(name, value);
            }
            response
        };
        let headers = response.headers_mut();
        tag.insert(headers);
        let (browser, cdn) = kind.policies(self.cdn_cache_mode);
        bitview_http::insert_cache_policy(headers, browser, cdn);
        Ok(Some(response))
    }
}

impl AppState {
    /// Answers a read. `resolve` runs on a worker once admitted by `gate` and within the read
    /// deadline, and whatever it pins, the answer's `build` reads under. An answer finding no
    /// body permit free waits for one off the worker, then is resolved again: nothing resolved
    /// before the wait is sent after it.
    pub async fn respond(
        &self,
        headers: &HeaderMap,
        gate: Gate<'_>,
        resolve: impl FnOnce(&Query) -> Result<Answer> + Clone + Send + 'static,
    ) -> Response {
        self.try_respond(headers, gate, resolve)
            .await
            .unwrap_or_else(|error| Error::from(error).into_response())
    }

    async fn try_respond(
        &self,
        headers: &HeaderMap,
        gate: Gate<'_>,
        resolve: impl FnOnce(&Query) -> Result<Answer> + Clone + Send + 'static,
    ) -> Result<Response> {
        let asked = Arc::new(Asked::new(self, headers, &gate));
        let deadline = self.query.read_deadline();
        let query = self.query.with_deadline(deadline);
        let again = gate.bodies.map(|bodies| (bodies, resolve.clone()));
        let first = Arc::clone(&asked);
        if let Some(response) = query
            .read_with_admission(gate.admission, move |q| first.respond(q, None, resolve(q)?))
            .await?
        {
            return Ok(response);
        }
        let (bodies, resolve) = again.ok_or(QueryError::Internal(
            "an answer without a body budget asked for a permit",
        ))?;
        let permit = timeout_at(Instant::from_std(deadline), RawBodyPermit::acquire(bodies))
            .await
            .map_err(|_| QueryError::ReadTimeout)??;
        query
            .read_with_admission(gate.admission, move |q| {
                asked.respond(q, Some(permit), resolve(q)?)
            })
            .await?
            .ok_or(QueryError::Internal(
                "an answer holding a body permit asked for one again",
            ))
    }

    /// A chain answer that names itself once resolved: `resolve` finds its source and says what
    /// identifies the representation (the block that fixes it, or its content while it can
    /// still change) and whether that is settled, which `settle` makes its kind; `build` then
    /// reads the source.
    #[cfg(feature = "full-api")]
    pub async fn respond_resolved<S: 'static, B: Into<Bytes>>(
        &self,
        headers: &HeaderMap,
        version: Version,
        content_type: &'static str,
        settle: Settle,
        resolve: impl FnOnce(&Query) -> Result<(S, RepresentationId, bool)>
        + Clone
        + Send
        + 'static,
        build: impl FnOnce(&Query, S) -> Result<B> + Clone + Send + 'static,
    ) -> Response {
        self.respond(headers, self.chain(), move |q| {
            let (source, identity, settled) = resolve(q)?;
            let tag = Etag::representation(version, identity);
            let answer = Answer::new(tag, move |q| {
                Ok(Content::new(content_type, build(q, source)?))
            });
            Ok(settle(answer, settled))
        })
        .await
    }

    /// A chain answer the query hands over as JSON with what names it: nothing cheaper does.
    #[cfg(feature = "full-api")]
    pub async fn respond_built<B: AsRef<[u8]> + Send + 'static>(
        &self,
        headers: &HeaderMap,
        built: impl FnOnce(&Query) -> Result<(B, RepresentationId)> + Clone + Send + 'static,
    ) -> Response {
        self.respond(headers, self.chain(), move |q| {
            let (bytes, identity) = built(q)?;
            // A body shared with the pool (built once per publication) isn't copied.
            let content = Content::new(JSON, Bytes::from_owner(bytes));
            Ok(Answer::ready(
                Etag::representation(Version::ONE, identity),
                content,
            ))
        })
        .await
    }

    /// Answers from memory on the caller's thread: only for an answer that reads no disk.
    pub fn respond_now(&self, headers: &HeaderMap, answer: Answer) -> Response {
        let asked = Asked::new(self, headers, &Gate::OPEN);
        match self.query.sync(|q| asked.respond(q, None, answer)) {
            Ok(Some(response)) => response,
            Ok(None) => Error::from(QueryError::Internal(
                "an answer without a body budget asked for a permit",
            ))
            .into_response(),
            Err(error) => Error::from(error).into_response(),
        }
    }
}
