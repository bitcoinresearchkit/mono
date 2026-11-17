use std::{borrow::Cow, path::Path, sync::Arc};

use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode, Uri, header},
    response::{IntoResponse, Redirect},
};
use bitview_http::{CachePolicy, IMMUTABLE, REVALIDATE};
use tokio::{sync::Semaphore, task::spawn_blocking};

use crate::{
    Error,
    page_info::Listing,
    router::Site,
    website::{File, RELEASES, WORKERS, hash},
};

/// Addresses served elsewhere on the site, each old folder (a path from the root) with where its addresses go now: a
/// link or a search result to the old address still arrives, and so does one written for another explorer's addresses.
/// Under a folder that lists its addresses (page info), only to one it lists.
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
    // Studio's presets' first addresses (October 2026), each to its place now, written as listed.
    ("studio/chart", To::Under("studio/presets")),
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
            None => not_found(
                wants_page(&headers)
                    .then(|| site.website.get_file(NOT_FOUND_PAGE).ok())
                    .flatten(),
            ),
        },
        answer => answer.into_response(),
    })
    .await
}

/// Where a request under a moved page's old folder goes now: the new page whatever follows the folder, or the same
/// place under the new folder when the address is written the one way (no other spelling names anything there).
/// From the path as sent (still percent-encoded), so what it encodes stays encoded in `Location`.
fn moved(path: &str, query: Option<&str>) -> Option<String> {
    let (to, rest) = MOVED.iter().find_map(|(from, to)| {
        let rest = path.strip_prefix('/')?.strip_prefix(from)?;
        (rest.is_empty() || rest.starts_with('/'))
            .then(|| (to, rest.strip_prefix('/').unwrap_or(rest)))
    })?;
    match to {
        To::Under(to) => is_one_address(path).then(|| with_query(&format!("/{to}/{rest}"), query)),
        To::Page(to) => Some(format!("/{to}/")),
    }
}

fn with_query(path: &str, query: Option<&str>) -> String {
    match query {
        Some(query) => format!("{path}?{query}"),
        None => path.to_owned(),
    }
}

/// The address written with the slash the other way.
fn other_spelling(address: &str) -> String {
    address
        .strip_suffix('/')
        .map_or_else(|| format!("{address}/"), str::to_owned)
}

/// A file's version, its content hash.
const VERSION: HeaderName = HeaderName::from_static("bitview-version");

/// For an address that names nothing: a page with 404 to a browser (`wants_page`), else the plain message (made a
/// problem document by the API server's error layer). Neither is stored: which one comes depends on `Accept`, which a
/// CDN's cache key leaves out.
fn not_found(page: Option<File>) -> Response<Body> {
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
    if let Some(release) = site.website.release()
        && let Some((asked, path)) = [RELEASES, WORKERS]
            .into_iter()
            .find_map(|folder| path.strip_prefix(folder))
            .and_then(|rest| rest.split_once('/'))
    {
        return released(site, path, asked == release, uri, request_headers);
    }
    let mut found = site.website.get_file(path)?;
    let file = &found.path;
    let content_type = found.content_type;
    let has_slash = path.ends_with('/');
    // A folder's page is at its address with the slash, the links' and the sitemap's.
    if !path.is_empty() && !has_slash && file.ends_with(Path::new(path).join("index.html")) {
        return Ok(permanent(&with_query(
            &format!("{}/", uri.path()),
            uri.query(),
        )));
    }
    // A slash after a file's name, or a query for anything but a page, whose script reads it.
    if (has_slash && file.ends_with(Path::new(path)))
        || (uri.query().is_some() && content_type != "text/html")
    {
        return Err(Error::not_found());
    }
    let varies = match site
        .pages
        .apply(&mut found, uri.path(), request_headers)
        .map_err(|error| Error::internal(format!("{}: {error}", uri.path())))?
    {
        Listing::Listed => true,
        Listing::Unlisted => false,
        // Not one of an app's addresses. One of them written the slash the other way, or under a folder that moved,
        // or both: there. Else a 404 for search engines; a browser still gets the app, to say what's missing there (a
        // renamed address).
        Listing::Unknown => {
            let moved = moved(uri.path(), None);
            let listed = [Some(uri.path()), moved.as_deref()]
                .into_iter()
                .flatten()
                .flat_map(|address| [address.to_owned(), other_spelling(address)])
                .find(|address| site.pages.lists(&found.path, address));
            return Ok(match listed {
                Some(to) => permanent(&with_query(&to, uri.query())),
                None => not_found(wants_page(request_headers).then_some(found)),
            });
        }
    };
    let policy = if varies {
        // Shared caches may ignore Vary: Accept. Browsers can still revalidate each representation.
        "private, max-age=0, must-revalidate"
    } else {
        REVALIDATE
    };
    let mut response = answer(found, policy, request_headers);
    if varies {
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Accept"));
    }
    Ok(response)
}

/// A file at a release's address (`RELEASES`, or a worker's: `WORKERS`), by its path written the one way (no folder's
/// page, no slash after a file's name, no page: those have one address, outside any release). The current release's
/// never changes, so it's held for good; another's (asked by a page from before a release) is the current file,
/// revalidated, as only the current release is served.
fn released(
    site: &Site,
    path: &str,
    current: bool,
    uri: &Uri,
    request_headers: &HeaderMap,
) -> Result<Response<Body>, Error> {
    let found = site.website.get_file(path)?;
    if found.path.to_str() != Some(path)
        || found.content_type == "text/html"
        || uri.query().is_some()
    {
        return Err(Error::not_found());
    }
    let policy = if current { IMMUTABLE } else { REVALIDATE };
    Ok(answer(found, policy, request_headers))
}

/// A file's 200, or its 304 to a request that has it.
fn answer(found: File, policy: &'static str, request_headers: &HeaderMap) -> Response<Body> {
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
    let mut response = if bitview_http::if_none_match(request_headers, etag.map(str::as_bytes)) {
        bitview_http::not_modified()
    } else {
        file_response(contents, content_type)
    };
    insert_caching(response.headers_mut(), etag, policy);
    response
}

/// The validator and cache policy a file's 200 and 304 share.
fn insert_caching(headers: &mut HeaderMap, etag: Option<&str>, policy: &'static str) {
    if let Some(etag) = etag {
        // Weak: a CDN compressing the file serves other bytes under the same validator.
        headers.insert(header::ETAG, format!("W/\"{etag}\"").parse().unwrap());
        // The same hash as the file's version, for an open page to tell a newer one is served
        // (`showcases/assets/version`): a CDN that rewrites pages drops their ETag (Cloudflare does), not this.
        headers.insert(VERSION, etag.parse().unwrap());
    }
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(policy));
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
