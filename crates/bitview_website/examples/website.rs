use std::time::Duration;

use axum::{
    ServiceExt,
    body::Body,
    http::{Request, Response, StatusCode, Uri},
    middleware::Next,
};
use bitview_website::{Website, router};
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer, classify::ServerErrorsFailureClass,
    compression::CompressionLayer, cors::CorsLayer, normalize_path::NormalizePathLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};
use tower_layer::Layer;
use tracing::{error, info};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = brk_logger::init(None);

    let website = Website::Default;

    if !website.is_enabled() {
        eprintln!("Website is disabled");
        return Ok(());
    }

    website.log();

    let compression_layer = CompressionLayer::new().br(true).gzip(true).zstd(true);

    let response_uri_layer = axum::middleware::from_fn(
        async |request: Request<Body>, next: Next| -> Response<Body> {
            let uri = request.uri().clone();
            let mut response = next.run(request).await;
            response.extensions_mut().insert(uri);
            response
        },
    );

    let trace_layer = TraceLayer::new_for_http()
        .on_request(())
        .on_response(
            |response: &Response<Body>, latency: Duration, _: &tracing::Span| {
                let status = response.status().as_u16();
                let Some(uri) = response.extensions().get::<Uri>() else {
                    return;
                };
                match response.status() {
                    StatusCode::OK
                    | StatusCode::NOT_MODIFIED
                    | StatusCode::TEMPORARY_REDIRECT
                    | StatusCode::PERMANENT_REDIRECT => info!(status, %uri, ?latency),
                    _ => error!(status, %uri, ?latency),
                }
            },
        )
        .on_body_chunk(())
        .on_failure(
            |error: ServerErrorsFailureClass, latency: Duration, _: &tracing::Span| {
                error!(?error, ?latency, "request failed");
            },
        )
        .on_eos(());

    let app = router(website)
        .layer(CatchPanicLayer::new())
        .layer(compression_layer)
        .layer(response_uri_layer)
        .layer(trace_layer)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(5),
        ))
        .layer(CorsLayer::permissive());

    let mut last_error = None;
    let (port, listener) = {
        let mut bound = None;

        for port in [3110, 3111] {
            match TcpListener::bind(("0.0.0.0", port)).await {
                Ok(listener) => {
                    bound = Some((port, listener));
                    break;
                }
                Err(error) => {
                    info!(port, ?error, "website server port unavailable");
                    last_error = Some(error);
                }
            }
        }

        bound.ok_or_else(|| last_error.expect("at least one port was attempted"))?
    };

    info!("website server listening on port {port}");

    let service = NormalizePathLayer::trim_trailing_slash().layer(app);

    axum::serve(
        listener,
        ServiceExt::<Request<Body>>::into_make_service(service),
    )
    .await
}
