use std::{borrow::Cow, path::Path, sync::Arc};

use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode, Uri, header},
    response::{IntoResponse, Redirect},
};
use bitview_http::{CDN_CACHE_CONTROL, CachePolicy, IMMUTABLE, REVALIDATE};
use tokio::{sync::Semaphore, task::spawn_blocking};

use crate::{
    Error,
    page_info::{self, Form, Listed, Listing},
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
    let found = match site.website.get_file(path) {
        // No file there: a listed page's Markdown twin, at an address of its own.
        Err(error) if error.is_not_found() => {
            return twin(site, uri, request_headers).ok_or(error);
        }
        found => found?,
    };
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
    match site.pages.find(&found.path, uri.path()) {
        Listing::Listed(listed) => {
            listed_answer(site, &found, &listed, Form::Html, uri.path(), request_headers)
        }
        Listing::Unlisted => Ok(answer(found, site.kept(Kept::Document), request_headers)),
        // Not one of an app's addresses. One it once had: where it opens now. One of them written the slash the other
        // way, or under a folder that moved, or both: there. Else a 404 for search engines; a browser still gets the
        // app, to say what's missing there (a renamed address).
        Listing::Unknown => {
            // (Either slash spelling of it, as a live one's.)
            let path = uri.path();
            let other = path.strip_suffix('/').map_or_else(|| format!("{path}/"), str::to_owned);
            let to = site.pages.moved(&found.path, path);
            if let Some(to) = to.or_else(|| site.pages.moved(&found.path, &other)) {
                // (The query before the target's fragment, where a URL keeps it.)
                let (to, fragment) = match to.split_once('#') {
                    Some((to, fragment)) => (to, format!("#{fragment}")),
                    None => (to, String::new()),
                };
                return Ok(permanent(&format!("{}{fragment}", with_query(to, uri.query()))));
            }
            let moved = moved(uri.path(), None);
            let listed = [Some(uri.path()), moved.as_deref()]
                .into_iter()
                .flatten()
                .flat_map(|address| [address.to_owned(), other_spelling(address)])
                .find(|address| site.pages.lists(&found.path, address));
            Ok(match listed {
                Some(to) => permanent(&with_query(&to, uri.query())),
                None => not_found(wants_page(request_headers).then_some(found)),
            })
        }
    }
}

/// The Markdown twin of a listed page, for an address that names no file: the page's address and `.md`
/// (`page_info::twin`), without a query. `None` when it is no listed page's.
fn twin(site: &Site, uri: &Uri, request_headers: &HeaderMap) -> Option<Response<Body>> {
    let page = page_info::twin_page(uri.path()).filter(|_| uri.query().is_none())?;
    let index = site.website.get_file(page.strip_prefix('/')?).ok()?;
    let Listing::Listed(listed) = site.pages.find(&index.path, page) else {
        return None;
    };
    listed_answer(site, &index, &listed, Form::Markdown, page, request_headers).ok()
}

/// A listed page at `address`, or its Markdown twin: each names the other in `Link`, so a reader of one finds the
/// other, and a search engine the page.
fn listed_answer(
    site: &Site,
    index: &File,
    listed: &Listed<'_>,
    form: Form,
    address: &str,
    request_headers: &HeaderMap,
) -> Result<Response<Body>, Error> {
    let contents = |listed: &Listed<'_>| {
        listed
            .contents(form, &index.contents)
            .map_err(|error| Error::internal(format!("{address}: {error}")))
    };
    let kept = site.kept(Kept::Document);
    let content_type = form.content_type();
    let mut response = match listed.tag(form) {
        // Named as the site loaded: a request that has it costs no page.
        Some(tag) if !cfg!(debug_assertions) => {
            if bitview_http::if_none_match(request_headers, Some(tag.as_bytes())) {
                tagged(bitview_http::not_modified(), Some(tag), kept)
            } else {
                tagged(file_response(contents(listed)?, content_type), Some(tag), kept)
            }
        }
        _ => conditional(contents(listed)?, None, content_type, kept, request_headers),
    };
    let link = match form {
        Form::Html => format!(
            "<{}>; rel=\"alternate\"; type=\"text/markdown\"",
            page_info::twin(address)
        ),
        Form::Markdown => format!("<{address}>; rel=\"canonical\""),
    };
    let link = HeaderValue::try_from(link).map_err(|error| Error::internal(error.to_string()))?;
    response.headers_mut().insert(header::LINK, link);
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
    let kept = site.kept(if current { Kept::Release } else { Kept::Document });
    Ok(answer(found, kept, request_headers))
}

/// How long an answer's bytes stay what they are at its address.
#[derive(Clone, Copy)]
enum Kept {
    /// At the current release's address: for good.
    Release,
    /// Anywhere else: until the next release of an embedded site, or any time for a folder's.
    Document,
}

impl Site {
    /// The browser's policy for an answer, and the CDN's own where it differs. Browsers are never purged, so a
    /// document is asked for again each second. A CDN purged on every release (`cdn_purged`, the server's `--cdn`)
    /// keeps an embedded site's documents until then, as only a release changes them.
    fn kept(&self, kept: Kept) -> (&'static str, Option<&'static str>) {
        match kept {
            Kept::Release => (IMMUTABLE, None),
            Kept::Document => {
                let fixed = self.cdn_purged && self.website.filesystem_path().is_none();
                (REVALIDATE, fixed.then_some(IMMUTABLE))
            }
        }
    }
}

/// A file's 200, or its 304 to a request that has it.
fn answer(
    found: File,
    kept: (&'static str, Option<&'static str>),
    request_headers: &HeaderMap,
) -> Response<Body> {
    conditional(found.contents, found.hash, found.content_type, kept, request_headers)
}

/// The 200 of `contents`, or the 304 to a request that has them, named by `known` or else by their hash.
fn conditional(
    contents: Bytes,
    known: Option<&str>,
    content_type: &'static str,
    kept: (&'static str, Option<&'static str>),
    request_headers: &HeaderMap,
) -> Response<Body> {
    // Release only. Embedded files carry theirs; filesystem content is mutable, so hash it now.
    let etag = (!cfg!(debug_assertions))
        .then(|| known.map_or_else(|| Cow::Owned(hash(&contents)), Cow::Borrowed));
    let etag = etag.as_deref();
    let response = if bitview_http::if_none_match(request_headers, etag.map(str::as_bytes)) {
        bitview_http::not_modified()
    } else {
        file_response(contents, content_type)
    };
    tagged(response, etag, kept)
}

/// A 200 or 304 with the validator and cache policy the two share.
fn tagged(
    mut response: Response<Body>,
    etag: Option<&str>,
    (browser, cdn): (&'static str, Option<&'static str>),
) -> Response<Body> {
    let headers = response.headers_mut();
    if let Some(etag) = etag {
        // Weak: a CDN compressing the file serves other bytes under the same validator.
        headers.insert(header::ETAG, format!("W/\"{etag}\"").parse().unwrap());
        // The same hash as the file's version, for an open page to tell a newer one is served
        // (`showcases/assets/version`): a CDN that rewrites pages drops their ETag (Cloudflare does), not this.
        headers.insert(VERSION, etag.parse().unwrap());
    }
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(browser));
    if let Some(cdn) = cdn {
        headers.insert(CDN_CACHE_CONTROL, HeaderValue::from_static(cdn));
    }
    response
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
