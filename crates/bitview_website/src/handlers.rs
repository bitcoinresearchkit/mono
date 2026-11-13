use std::{borrow::Cow, path::Path};

use axum::{
    body::Body,
    extract::{Path as ExtractPath, State},
    http::{HeaderMap, HeaderName, HeaderValue, Response, Uri, header},
    response::{IntoResponse, Redirect},
};
use tokio::{sync::Semaphore, task::spawn_blocking};

use crate::{
    Error, Website,
    website::{File, hash},
};

/// Pages that moved, each old folder with its new one: a link or a search result to the old address still arrives.
const MOVED: &[(&str, &str)] = &[("series", "catalog"), ("api-changes", "changes")];

pub async fn file_handler(
    State(website): State<Website>,
    headers: HeaderMap,
    uri: Uri,
    ExtractPath(path): ExtractPath<String>,
) -> Result<Response<Body>, Error> {
    if !is_one_address(uri.path()) {
        return Err(not_found());
    }
    if let Some(to) = moved(uri.path(), uri.query()) {
        return Ok(Redirect::permanent(&to).into_response());
    }
    serve(website, path, uri, headers).await
}

/// Where a request under a moved page's old folder goes now: the same place under the new one, its query kept.
/// From the path as sent (still percent-encoded), so what it encodes stays encoded in `Location`.
pub(crate) fn moved(path: &str, query: Option<&str>) -> Option<String> {
    let path = path.trim_start_matches('/');
    let (folder, rest) = path.split_once('/').unwrap_or((path, ""));
    let (_, to) = MOVED.iter().find(|(from, _)| *from == folder)?;
    let query = query.map(|query| format!("?{query}")).unwrap_or_default();
    Some(format!("/{to}/{rest}{query}"))
}

pub async fn index_handler(
    State(website): State<Website>,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Error> {
    serve(website, String::new(), uri, headers).await
}

/// A file's version, its content hash.
const VERSION: HeaderName = HeaderName::from_static("bitview-version");

/// Concurrent filesystem responses.
static FILESYSTEM_READS: Semaphore = Semaphore::const_new(2);

/// Filesystem reads block, so they leave the executor, each holding a permit until its job ends.
async fn serve(
    website: Website,
    path: String,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response<Body>, Error> {
    if website.filesystem_path().is_none() {
        return respond(&website, &path, &uri, &headers);
    }
    let permit = FILESYSTEM_READS
        .acquire()
        .await
        .map_err(|e| Error::internal(e.to_string()))?;
    spawn_blocking(move || {
        let _permit = permit;
        respond(&website, &path, &uri, &headers)
    })
    .await
    .map_err(|e| Error::internal(e.to_string()))?
}

fn respond(
    website: &Website,
    path: &str,
    uri: &Uri,
    request_headers: &HeaderMap,
) -> Result<Response<Body>, Error> {
    let File {
        path: file,
        contents,
        hash: file_hash,
        content_type,
    } = website.get_file(path)?;
    let has_slash = path.ends_with('/');
    // A folder's page is at its address with the slash, the links' and the sitemap's.
    if !path.is_empty() && !has_slash && file.ends_with(Path::new(path).join("index.html")) {
        let query = uri.query().map(|query| format!("?{query}")).unwrap_or_default();
        return Ok(Redirect::permanent(&format!("{}/{query}", uri.path())).into_response());
    }
    // A slash after a file's name, or a query for anything but a page, whose script reads it.
    if (has_slash && file.ends_with(Path::new(path)))
        || (uri.query().is_some() && content_type != "text/html")
    {
        return Err(not_found());
    }
    // Release only. Embedded files carry theirs; filesystem content is mutable, so hash it now.
    let etag = (!cfg!(debug_assertions))
        .then(|| file_hash.map_or_else(|| Cow::Owned(hash(&contents)), Cow::Borrowed));
    let etag = etag.as_deref();
    if bitview_http::if_none_match(request_headers, etag.map(str::as_bytes)) {
        let mut response = bitview_http::not_modified();
        insert_caching(response.headers_mut(), etag);
        return Ok(response);
    }
    let mut response = Response::new(Body::from(contents));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    insert_caching(headers, etag);

    Ok(response)
}

/// The validator and cache policy a file's 200 and 304 share: every file revalidates.
fn insert_caching(headers: &mut HeaderMap, etag: Option<&str>) {
    if let Some(etag) = etag {
        // Weak: a CDN compressing the file serves other bytes under the same validator.
        headers.insert(header::ETAG, format!("W/\"{etag}\"").parse().unwrap());
        // The same hash as the file's version, for an open page to tell a newer one is served
        // (`showcases/assets/version`): a CDN that rewrites pages drops their ETag (Cloudflare does), not this.
        headers.insert(VERSION, etag.parse().unwrap());
    }
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=1, must-revalidate"),
    );
}

/// Whether the path as sent is written the one way: no escape, no empty, `.` or `..` segment, which
/// also keeps a request inside the website, and no `index.html`, as a folder's page is at the
/// folder's address.
fn is_one_address(path: &str) -> bool {
    let path = path.trim_prefix('/');
    !path.contains('%')
        && path
            .strip_suffix('/')
            .unwrap_or(path)
            .split('/')
            .all(|segment| !matches!(segment, "" | "." | ".." | "index.html"))
}

fn not_found() -> Error {
    Error::not_found("File not found")
}
