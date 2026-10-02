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

#[cfg(test)]
mod tests {
    use std::{fs, future::Future, sync::OnceLock};

    use axum::{
        body::{Body, to_bytes},
        http::{HeaderMap, Response, StatusCode, header},
        response::IntoResponse,
    };
    use tempfile::tempdir;
    use tokio::runtime::{Builder, Runtime};

    use super::serve;
    use crate::Website;

    fn block_on<F: Future>(future: F) -> F::Output {
        static RUNTIME: OnceLock<Runtime> = OnceLock::new();

        RUNTIME
            .get_or_init(|| Builder::new_current_thread().enable_all().build().unwrap())
            .block_on(future)
    }

    fn body_bytes(response: Response<Body>) -> Vec<u8> {
        block_on(async {
            to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec()
        })
    }

    #[test]
    fn index_etag_never_validates_other_or_missing_html() {
        let root = serve(&Website::Default, "", &HeaderMap::new()).unwrap();
        let mut headers = HeaderMap::new();
        if let Some(etag) = root.headers().get(header::ETAG) {
            headers.insert(header::IF_NONE_MATCH, etag.clone());
        }
        for path in ["", "index.html", "charts/price"] {
            let response = serve(&Website::Default, path, &headers).unwrap();
            assert_eq!(
                response.status(),
                if cfg!(debug_assertions) {
                    StatusCode::OK
                } else {
                    StatusCode::NOT_MODIFIED
                }
            );
        }
        let standalone = serve(&Website::Default, "assets/logo/demo.html", &headers).unwrap();
        assert_eq!(standalone.status(), StatusCode::OK);
        if let Some(etag) = standalone.headers().get(header::ETAG) {
            assert_ne!(Some(etag), headers.get(header::IF_NONE_MATCH));
            let mut own_headers = HeaderMap::new();
            own_headers.insert(header::IF_NONE_MATCH, etag.clone());
            let response = serve(&Website::Default, "assets/logo/demo.html", &own_headers).unwrap();
            assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
            assert!(body_bytes(response).is_empty());
        }
        assert!(!body_bytes(standalone).is_empty());
        for (website, path) in [
            (Website::Default, "missing-page.html"),
            (Website::Disabled, ""),
        ] {
            assert_eq!(
                serve(&website, path, &headers)
                    .unwrap_err()
                    .into_response()
                    .status(),
                StatusCode::NOT_FOUND
            );
        }
    }

    #[test]
    fn filesystem_indexes_are_independent_and_revalidated_from_disk() {
        let first = tempdir().unwrap();
        let second = tempdir().unwrap();
        let first_site = Website::Filesystem(first.path().to_owned());
        let second_site = Website::Filesystem(second.path().to_owned());
        let first_path = first.path().join("index.html");
        fs::write(&first_path, "first site").unwrap();
        fs::write(second.path().join("index.html"), "second site").unwrap();

        let embedded = serve(&Website::Default, "", &HeaderMap::new()).unwrap();
        let mut headers = HeaderMap::new();
        if let Some(etag) = embedded.headers().get(header::ETAG) {
            headers.insert(header::IF_NONE_MATCH, etag.clone());
        }
        for (site, expected) in [(&first_site, "first site"), (&second_site, "second site")] {
            let response = serve(site, "", &headers).unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            if let Some(etag) = response.headers().get(header::ETAG) {
                assert_ne!(Some(etag), headers.get(header::IF_NONE_MATCH));
                let mut own_headers = HeaderMap::new();
                own_headers.insert(header::IF_NONE_MATCH, etag.clone());
                assert_eq!(
                    serve(site, "", &own_headers).unwrap().status(),
                    StatusCode::NOT_MODIFIED
                );
            }
            assert_eq!(body_bytes(response), expected.as_bytes());
        }
        let previous = serve(&first_site, "", &headers).unwrap();
        if let Some(etag) = previous.headers().get(header::ETAG) {
            headers.insert(header::IF_NONE_MATCH, etag.clone());
        }
        fs::write(&first_path, "updated site").unwrap();
        assert_eq!(
            body_bytes(serve(&first_site, "", &headers).unwrap()),
            b"updated site"
        );
        fs::remove_file(&first_path).unwrap();
        assert_eq!(
            serve(&first_site, "", &headers)
                .unwrap_err()
                .into_response()
                .status(),
            StatusCode::NOT_FOUND
        );
    }
}
