use std::{
    borrow::Cow,
    convert::Infallible,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::{Body, Bytes, HttpBody},
    extract::{Path as ExtractPath, State},
    http::{HeaderMap, Response, StatusCode},
};
use http_body::Frame;
use importmap::ImportMap;
use tokio::{sync::Semaphore, task::spawn_blocking};

use crate::{Error, HeaderMapExtended, Website, website::File};

pub async fn file_handler(
    State(website): State<Website>,
    headers: HeaderMap,
    ExtractPath(path): ExtractPath<String>,
) -> Result<Response<Body>, Error> {
    serve(website, path, headers).await
}

pub async fn index_handler(
    State(website): State<Website>,
    headers: HeaderMap,
) -> Result<Response<Body>, Error> {
    serve(website, String::new(), headers).await
}

/// Concurrent filesystem responses: a release index rescans and hashes the whole folder.
static FILESYSTEM_READS: Semaphore = Semaphore::const_new(2);

/// Filesystem reads (and the release importmap scan) block, so they leave the executor, each
/// holding a permit until its job ends.
async fn serve(
    website: Website,
    path: String,
    headers: HeaderMap,
) -> Result<Response<Body>, Error> {
    if website.filesystem_path().is_none() {
        return respond(&website, &path, &headers);
    }
    let permit = FILESYSTEM_READS
        .acquire()
        .await
        .map_err(|e| Error::internal(e.to_string()))?;
    spawn_blocking(move || {
        let _permit = permit;
        respond(&website, &path, &headers)
    })
    .await
    .map_err(|e| Error::internal(e.to_string()))?
}

fn respond(
    website: &Website,
    path: &str,
    request_headers: &HeaderMap,
) -> Result<Response<Body>, Error> {
    let path = sanitize(path);

    let is_html =
        path.is_empty() || Path::new(&path).extension().is_none() || path.ends_with(".html");

    let File { contents, hash } = website.get_file(&path)?;
    // Release only. Embedded files carry theirs; filesystem content is mutable, so hash it now.
    let hash = (!cfg!(debug_assertions))
        .then(|| hash.map_or_else(|| Cow::Owned(ImportMap::hash(&contents)), Cow::Borrowed));
    // Only a URL naming its content's hash is immutable; fixed names (JSON, workers, fonts) and
    // stale hashes revalidate.
    let immutable = !is_html
        && hash
            .as_deref()
            .is_some_and(|hash| ImportMap::names_hash(Path::new(&path), hash));
    // Weak: compression serves other bytes under the same validator.
    let etag = hash
        .filter(|_| !immutable)
        .map(|hash| format!("W/\"{hash}\""));
    if let Some(etag) = etag.as_deref()
        && request_headers.has_etag(etag)
    {
        return Ok(not_modified(etag));
    }
    let mut response = Response::new(Body::from(contents));
    let headers = response.headers_mut();

    if is_html {
        headers.insert_content_type_text_html();
    } else {
        headers.insert_content_type(Path::new(&path));
    }
    if let Some(etag) = etag.as_deref() {
        headers.insert_etag(etag);
    }
    headers.insert_vary_accept_encoding();

    if immutable {
        headers.insert_cache_control_immutable();
    } else {
        headers.insert_cache_control_must_revalidate();
    }

    Ok(response)
}

/// Without a length: a 304's Content-Length would have to describe the full response.
fn not_modified(etag: &str) -> Response<Body> {
    let mut response = Response::new(Body::new(NotModifiedBody));
    *response.status_mut() = StatusCode::NOT_MODIFIED;
    let headers = response.headers_mut();
    headers.insert_etag(etag);
    headers.insert_vary_accept_encoding();
    headers.insert_cache_control_must_revalidate();
    response
}

/// Empty content without the exact-length hint Axum turns into `Content-Length: 0`.
struct NotModifiedBody;

impl HttpBody for NotModifiedBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Infallible>>> {
        Poll::Ready(None)
    }

    fn is_end_stream(&self) -> bool {
        true
    }
}

/// Sanitize path to prevent directory traversal attacks
fn sanitize(path: &str) -> String {
    path.split('/')
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect::<Vec<_>>()
        .join("/")
}
