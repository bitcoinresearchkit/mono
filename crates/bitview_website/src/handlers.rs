use std::{borrow::Cow, path::Path};

use axum::{
    body::Body,
    extract::{Path as ExtractPath, State},
    http::{HeaderMap, Response, StatusCode},
};
use importmap::ImportMap;
use tokio::task::spawn_blocking;

use crate::{
    Error, HeaderMapExtended, Website,
    website::{File, content_etag},
};

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

/// Filesystem reads (and the release importmap scan) block, so they leave the executor.
async fn serve(
    website: Website,
    path: String,
    headers: HeaderMap,
) -> Result<Response<Body>, Error> {
    if website.filesystem_path().is_none() {
        return respond(&website, &path, &headers);
    }
    spawn_blocking(move || respond(&website, &path, &headers))
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

    let File { contents, etag } = website.get_file(&path)?;
    // Release only. Embedded files carry theirs; filesystem content is mutable, so hash it now.
    let etag = (!cfg!(debug_assertions))
        .then(|| etag.map_or_else(|| Cow::Owned(content_etag(&contents)), Cow::Borrowed));
    // Only a URL naming its content's hash is immutable; fixed names (JSON, workers, fonts) and
    // stale hashes revalidate.
    let immutable = !is_html
        && etag
            .as_deref()
            .is_some_and(|etag| ImportMap::names_hash(Path::new(&path), etag.trim_matches('"')));
    let etag = etag.filter(|_| !immutable);
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

    if immutable {
        headers.insert_cache_control_immutable();
    } else {
        headers.insert_cache_control_must_revalidate();
    }

    Ok(response)
}

fn not_modified(etag: &str) -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NOT_MODIFIED;
    let headers = response.headers_mut();
    headers.insert_etag(etag);
    headers.insert_cache_control_must_revalidate();
    response
}

/// Sanitize path to prevent directory traversal attacks
fn sanitize(path: &str) -> String {
    path.split('/')
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect::<Vec<_>>()
        .join("/")
}
