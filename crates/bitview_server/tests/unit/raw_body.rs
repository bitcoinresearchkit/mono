use std::{
    convert::Infallible,
    future::{poll_fn, ready},
    pin::Pin,
};

use axum::{
    body::HttpBody,
    http::{
        Request,
        header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE},
    },
};
use tower::{ServiceExt, service_fn};
use tower_layer::Layer;

use super::*;
use crate::compression_layer;

#[tokio::test]
async fn compression_releases_input_without_releasing_response_admission() {
    for encoding in ["identity", "gzip", "br", "zstd"] {
        for compressible in [true, false] {
            let budget = Arc::new(Semaphore::new(1));
            let permit = RawBodyPermit::try_acquire(&budget).unwrap();
            let mut random = 1u32;
            let input: Arc<[u8]> = (0..128 * 1024)
                .map(|_| {
                    random ^= random << 13;
                    random ^= random >> 17;
                    random ^= random << 5;
                    if compressible { 0 } else { random as u8 }
                })
                .collect();
            let input_lifetime = Arc::downgrade(&input);
            let response =
                permit.response(CacheParams::deploy(), Bytes::from_owner(input), |headers| {
                    headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());
                });
            let mut response = Some(response);
            let service = compression_layer().layer(service_fn(move |_: Request<Body>| {
                ready(Ok::<_, Infallible>(response.take().unwrap()))
            }));
            let request = Request::builder()
                .header(ACCEPT_ENCODING, encoding)
                .body(Body::empty())
                .unwrap();
            let response = service.oneshot(request).await.unwrap().map(Body::new);
            assert_eq!(
                response
                    .headers()
                    .get(CONTENT_ENCODING)
                    .map(|v| v.to_str().unwrap()),
                if encoding == "identity" {
                    None
                } else {
                    Some(encoding)
                }
            );
            let mut body = RawBodyPermit::retain(response).into_body();
            let mut frames = Vec::new();
            while let Some(frame) = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await {
                if let Ok(bytes) = frame.unwrap().into_data() {
                    frames.push(bytes);
                }
                assert_eq!(budget.available_permits(), 0);
            }
            assert!(!frames.is_empty());
            // Some codecs retain an exhausted input chunk until their body
            // is dropped. Output-frame admission must survive either case.
            drop(body);
            if encoding != "identity" {
                assert!(
                    input_lifetime.upgrade().is_none(),
                    "{encoding} retained consumed input"
                );
            }
            assert_eq!(
                budget.available_permits(),
                0,
                "{encoding} released before its output frames"
            );
            let last = frames.pop().unwrap();
            drop(frames);
            assert_eq!(budget.available_permits(), 0);
            drop(last);
            assert_eq!(budget.available_permits(), 1);
            assert!(input_lifetime.upgrade().is_none());
        }
    }
}
