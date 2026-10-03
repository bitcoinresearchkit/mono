use std::path::Path;

use axum::{
    body::Body,
    extract::{Path as ExtractPath, State},
    http::{HeaderMap, Response, StatusCode},
};

use crate::{Error, HeaderMapExtended, Website, website::content_etag};

pub async fn file_handler(
    State(website): State<Website>,
    headers: HeaderMap,
    path: ExtractPath<String>,
) -> Result<Response<Body>, Error> {
    serve(&website, &path.0, &headers)
}

pub async fn index_handler(
    State(website): State<Website>,
    headers: HeaderMap,
) -> Result<Response<Body>, Error> {
    serve(&website, "", &headers)
}

fn serve(
    website: &Website,
    path: &str,
    request_headers: &HeaderMap,
) -> Result<Response<Body>, Error> {
    let path = sanitize(path);

    let is_html =
        path.is_empty() || Path::new(&path).extension().is_none() || path.ends_with(".html");

    // Etag 304 check (release mode, HTML only)
    if is_html
        && let Some(etag) = website.index_etag_for(&path)
        && request_headers.has_etag(etag)
    {
        return Ok(not_modified(etag));
    }

    let content = website.get_file(&path)?;
    // Mutable files and standalone HTML must be read before their validator is
    // known. Only the immutable embedded index can use the early path above.
    let etag = (!cfg!(debug_assertions) && is_html).then(|| {
        website
            .index_etag_for(&path)
            .map(str::to_owned)
            .unwrap_or_else(|| content_etag(&content))
    });
    if let Some(etag) = etag.as_deref()
        && request_headers.has_etag(etag)
    {
        return Ok(not_modified(etag));
    }
    let mut response = Response::new(Body::from(content));
    let headers = response.headers_mut();

    if is_html {
        headers.insert_content_type_text_html();
        if let Some(etag) = etag.as_deref() {
            headers.insert_etag(etag);
        }
    } else {
        headers.insert_content_type(Path::new(&path));
    }

    if cfg!(debug_assertions) || is_html {
        headers.insert_cache_control_must_revalidate();
    } else {
        headers.insert_cache_control_immutable();
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
