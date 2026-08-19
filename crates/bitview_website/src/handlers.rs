use std::path::Path;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Response, StatusCode},
};

use crate::{Error, HeaderMapExtended, Website};

pub async fn file_handler(
    State(website): State<Website>,
    headers: HeaderMap,
    path: axum::extract::Path<String>,
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
        && let Some(etag) = website.index_etag()
        && request_headers.has_etag(etag)
    {
        let mut response = Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .unwrap();
        let headers = response.headers_mut();
        headers.insert_etag(etag);
        headers.insert_cache_control_must_revalidate();
        return Ok(response);
    }

    let content = website.get_file(&path)?;
    let mut response = Response::new(Body::from(content));
    let headers = response.headers_mut();

    if is_html {
        headers.insert_content_type_text_html();
        if let Some(etag) = website.index_etag() {
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

/// Sanitize path to prevent directory traversal attacks
fn sanitize(path: &str) -> String {
    path.split('/')
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use std::{future::Future, sync::OnceLock};

    use axum::{
        body::{Body, to_bytes},
        http::{HeaderMap, StatusCode, header},
        response::IntoResponse,
    };

    use super::{sanitize, serve};
    use crate::Website;

    fn block_on<F: Future>(future: F) -> F::Output {
        static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

        RUNTIME
            .get_or_init(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
            })
            .block_on(future)
    }

    fn body_bytes(response: axum::http::Response<Body>) -> Vec<u8> {
        block_on(async {
            to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec()
        })
    }

    #[test]
    fn sanitize_removes_empty_and_traversal_segments() {
        assert_eq!(sanitize("../styles/reset.css"), "styles/reset.css");
        assert_eq!(sanitize("./styles//reset.css"), "styles/reset.css");
        assert_eq!(sanitize(""), "");
    }

    #[test]
    fn serves_index_html_for_root_and_spa_routes() {
        let request_headers = HeaderMap::new();

        let root = serve(&Website::Default, "", &request_headers).unwrap();
        assert_eq!(root.status(), StatusCode::OK);
        assert_eq!(
            root.headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap(),
            "text/html",
        );
        assert_eq!(
            root.headers()
                .get(header::CACHE_CONTROL)
                .unwrap()
                .to_str()
                .unwrap(),
            "public, max-age=1, must-revalidate",
        );

        if cfg!(debug_assertions) {
            assert!(root.headers().get(header::ETAG).is_none());
        } else {
            assert!(root.headers().get(header::ETAG).is_some());
        }

        let root_body = body_bytes(root);
        let spa_body =
            body_bytes(serve(&Website::Default, "charts/price", &request_headers).unwrap());

        assert_eq!(root_body, spa_body);
        assert!(
            String::from_utf8(root_body)
                .unwrap()
                .contains("<!doctype html>")
        );
    }

    #[test]
    fn serves_static_assets_with_expected_headers() {
        let response = serve(&Website::Default, "styles/reset.css", &HeaderMap::new()).unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap(),
            "text/css",
        );

        let expected_cache_control = if cfg!(debug_assertions) {
            "public, max-age=1, must-revalidate"
        } else {
            "public, max-age=31536000, immutable"
        };

        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .unwrap()
                .to_str()
                .unwrap(),
            expected_cache_control,
        );
        assert!(!body_bytes(response).is_empty());
    }

    #[test]
    fn traversal_like_paths_resolve_to_sanitized_assets() {
        let direct =
            body_bytes(serve(&Website::Default, "styles/reset.css", &HeaderMap::new()).unwrap());
        let sanitized =
            body_bytes(serve(&Website::Default, "../styles/reset.css", &HeaderMap::new()).unwrap());

        assert_eq!(sanitized, direct);
    }

    #[test]
    fn missing_files_with_extensions_return_not_found() {
        let response = serve(&Website::Default, "styles/missing.css", &HeaderMap::new())
            .unwrap_err()
            .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
