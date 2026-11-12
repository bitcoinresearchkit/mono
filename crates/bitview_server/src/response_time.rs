use std::time::Instant;

use axum::{
    body::Body,
    http::{Method, Request, Response, header::ETAG},
    middleware::Next,
};
use bitview_http::CachePolicy;
use tracing::{error, info};

pub(crate) async fn respond(request: Request<Body>, next: Next) -> Response<Body> {
    let uri = request.uri().clone();
    let method = request.method().clone();
    let start = Instant::now();
    let mut response = next.run(request).await;
    let latency = start.elapsed();
    let status_code = response.status();
    let status = status_code.as_u16();

    if status_code.is_informational() || status_code.is_success() || status_code.is_redirection() {
        info!(%method, status, %uri, ?latency);
    } else {
        error!(%method, status, %uri, ?latency);
    }

    response.headers_mut().insert(
        "X-Response-Time",
        format!("{}us", latency.as_micros()).parse().unwrap(),
    );
    if method == Method::POST {
        CachePolicy::NoStore.apply(response.headers_mut());
        response.headers_mut().remove(ETAG);
    }
    response
}
