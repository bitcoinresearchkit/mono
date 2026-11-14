use std::{borrow::Cow, path::Path, sync::Arc};

use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode, Uri, header},
    response::{IntoResponse, Redirect},
};
use bitview_http::{CachePolicy, REVALIDATE};
use tokio::{sync::Semaphore, task::spawn_blocking};

use crate::{
    Error, Website,
    router::Site,
    website::{File, hash},
};

/// Addresses served elsewhere on the site, each old folder with where its addresses go now: a link or a search
/// result to the old address still arrives, and so does one written for another explorer's addresses.
const MOVED: &[(&str, To)] = &[
    ("series", To::Under("catalog")),
    ("api-changes", To::Under("changes")),
    // The explorer's places (its routes, `parse()` in showcases/explorer/index.html), at the root on mempool.space and
    // the explorers that copy its addresses.
    ("address", To::Under("explorer/address")),
    ("tx", To::Under("explorer/tx")),
    ("block", To::Under("explorer/block")),
    ("mempool-block", To::Under("explorer/mempool-block")),
    // The old charts' addresses name nothing in Studio, which replaced them.
    ("charts", To::Page("studio")),
];

/// Where a moved folder's addresses go.
enum To {
    /// The same place under the new folder, the query kept.
    Under(&'static str),
    /// The new folder's page.
    Page(&'static str),
}

/// The site's page for an address that names nothing.
const NOT_FOUND_PAGE: &str = "404.html";

/// Any address on the site, `/` included: the file it names, else where it moved, else the not-found answer, in one job
/// on the website's files. A file there always wins, as a site served from a folder may have its own.
pub async fn file_handler(
    State(site): State<Arc<Site>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    read(site, move |site| match respond(site, &uri, &headers) {
        Err(error) if error.is_not_found() => match moved(uri.path(), uri.query()) {
            Some(to) => permanent(&to),
            None => not_found(&site.website, &headers),
        },
        answer => answer.into_response(),
    })
    .await
}

/// Where a request under a moved page's old folder goes now: the new page whatever follows the folder, or the same
/// place under the new folder when the address is written the one way (no other spelling names anything there).
/// From the path as sent (still percent-encoded), so what it encodes stays encoded in `Location`.
fn moved(path: &str, query: Option<&str>) -> Option<String> {
    let trimmed = path.trim_start_matches('/');
    let (folder, rest) = trimmed.split_once('/').unwrap_or((trimmed, ""));
    let (_, to) = MOVED.iter().find(|(from, _)| *from == folder)?;
    match to {
        To::Under(to) => is_one_address(path).then(|| {
            let query = query.map(|query| format!("?{query}")).unwrap_or_default();
            format!("/{to}/{rest}{query}")
        }),
        To::Page(to) => Some(format!("/{to}/")),
    }
}

/// A file's version, its content hash.
const VERSION: HeaderName = HeaderName::from_static("bitview-version");

/// For an address that names nothing: the site's page for it to a browser, with 404, else the plain message (made a
/// problem document by the API server's error layer). Neither is stored: which one comes depends on `Accept`, which a
/// CDN's cache key leaves out.
fn not_found(website: &Website, headers: &HeaderMap) -> Response<Body> {
    let page = if wants_page(headers) {
        website.get_file(NOT_FOUND_PAGE).ok()
    } else {
        None
    };
    let mut response = match page {
        Some(page) => {
            let mut response = file_response(page.contents, page.content_type);
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
        }
        None => Error::not_found().into_response(),
    };
    CachePolicy::NoStore.mark(&mut response);
    response
}

fn file_response(contents: Bytes, content_type: &'static str) -> Response<Body> {
    let mut response = Response::new(Body::from(contents));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response
}

/// Whether a request asks for a page: a browser's navigation lists `text/html` in `Accept`.
fn wants_page(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::ACCEPT)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|range| bitview_http::media_type(range).eq_ignore_ascii_case("text/html"))
}

/// Concurrent filesystem jobs.
static FILESYSTEM_READS: Semaphore = Semaphore::const_new(2);

/// Runs a job on the website's files. Filesystem reads block, so there it leaves the executor, holding a permit until it
/// ends.
async fn read(
    site: Arc<Site>,
    job: impl FnOnce(&Site) -> Response<Body> + Send + 'static,
) -> Response<Body> {
    if site.website.filesystem_path().is_none() {
        return job(&site);
    }
    let permit = match FILESYSTEM_READS.acquire().await {
        Ok(permit) => permit,
        Err(e) => return Error::internal(e.to_string()).into_response(),
    };
    spawn_blocking(move || {
        let _permit = permit;
        job(&site)
    })
    .await
    .unwrap_or_else(|e| Error::internal(e.to_string()).into_response())
}

/// A 308 to where an address is answered now, revalidated as the site's files are: without a lifetime a browser may
/// keep a permanent redirect as long as it likes, and where an address goes can change with a release (`MOVED`).
fn permanent(location: &str) -> Response<Body> {
    let mut response = Redirect::permanent(location).into_response();
    CachePolicy::Revalidate.apply(response.headers_mut());
    response
}

fn respond(site: &Site, uri: &Uri, request_headers: &HeaderMap) -> Result<Response<Body>, Error> {
    // The path as sent: written the one way, it has no escape to decode. The not-found page is
    // the answer to other addresses, not one of its own.
    let path = uri.path().strip_prefix('/').unwrap_or_default();
    if !is_one_address(uri.path()) || path == NOT_FOUND_PAGE {
        return Err(Error::not_found());
    }
    let mut found = site.website.get_file(path)?;
    let file = &found.path;
    let content_type = found.content_type;
    let has_slash = path.ends_with('/');
    // A folder's page is at its address with the slash, the links' and the sitemap's.
    if !path.is_empty() && !has_slash && file.ends_with(Path::new(path).join("index.html")) {
        let query = uri
            .query()
            .map(|query| format!("?{query}"))
            .unwrap_or_default();
        return Ok(permanent(&format!("{}/{query}", uri.path())));
    }
    // A slash after a file's name, or a query for anything but a page, whose script reads it.
    if (has_slash && file.ends_with(Path::new(path)))
        || (uri.query().is_some() && content_type != "text/html")
    {
        return Err(Error::not_found());
    }
    let varies = site
        .pages
        .apply(&mut found, uri.path(), request_headers)
        .map_err(|error| Error::internal(format!("{}: {error}", uri.path())))?;
    let File {
        contents,
        hash: file_hash,
        content_type,
        ..
    } = found;
    // Release only. Embedded files carry theirs; filesystem content is mutable, so hash it now.
    let etag = (!cfg!(debug_assertions))
        .then(|| file_hash.map_or_else(|| Cow::Owned(hash(&contents)), Cow::Borrowed));
    let etag = etag.as_deref();
    if bitview_http::if_none_match(request_headers, etag.map(str::as_bytes)) {
        let mut response = bitview_http::not_modified();
        insert_caching(response.headers_mut(), etag, varies);
        return Ok(response);
    }
    let mut response = file_response(contents, content_type);
    insert_caching(response.headers_mut(), etag, varies);
    Ok(response)
}

/// The validator and cache policy a file's 200 and 304 share: every file revalidates.
fn insert_caching(headers: &mut HeaderMap, etag: Option<&str>, varies: bool) {
    if varies {
        headers.insert(header::VARY, HeaderValue::from_static("Accept"));
    }
    if let Some(etag) = etag {
        // Weak: a CDN compressing the file serves other bytes under the same validator.
        headers.insert(header::ETAG, format!("W/\"{etag}\"").parse().unwrap());
        // The same hash as the file's version, for an open page to tell a newer one is served
        // (`showcases/assets/version`): a CDN that rewrites pages drops their ETag (Cloudflare does), not this.
        headers.insert(VERSION, etag.parse().unwrap());
    }
    // Shared caches may ignore Vary: Accept. Browsers can still revalidate each representation.
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if varies {
            "private, max-age=0, must-revalidate"
        } else {
            REVALIDATE
        }),
    );
}

/// Whether the path as sent is written the one way: no escape, no empty, `.` or `..` segment, which
/// also keeps a request inside the website, and no `index.html`, as a folder's page is at the
/// folder's address.
fn is_one_address(path: &str) -> bool {
    let path = path.trim_prefix('/');
    !path.contains('%')
        && path
            .split_terminator('/')
            .all(|segment| !matches!(segment, "" | "." | ".." | "index.html"))
}
