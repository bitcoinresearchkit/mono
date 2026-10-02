use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use axum::{
    body::{self, Bytes},
    http::{HeaderMap, StatusCode, header::IF_NONE_MATCH},
};
use bitview_query::RepresentationId;
use brk_types::Version;
use tokio::{
    spawn,
    sync::oneshot,
    task,
    time::{self, timeout},
};

use super::chain_fixture::run;
use crate::{CacheParams, CacheStrategy};

#[test]
fn response_capacity_wait_releases_snapshot_and_resolves_again() {
    use crate::raw_body::RawBodyPermit;

    use std::sync::atomic::{AtomicUsize, Ordering};
    run(|state, _| async move {
        let bodies = state.raw_block_bodies.clone();
        let held = bodies
            .clone()
            .acquire_many_owned(bodies.available_permits() as u32)
            .await
            .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let reader = state.clone();
        let pending = spawn(async move {
            let budget = bodies.clone();
            reader
                .read_body(&reader.sync_query, &bodies, move |q, permit| {
                    let _snapshot = q.resolve_blocks_v1(None, 10)?;
                    let revision = q.indexer().publication().revision();
                    count.fetch_add(1, Ordering::SeqCst);
                    let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&budget))
                    else {
                        return Ok(None);
                    };
                    Ok(Some(permit.response(
                        CacheParams::deploy(),
                        revision.to_string().into(),
                        |_| {},
                    )))
                })
                .await
                .unwrap()
        });
        timeout(Duration::from_secs(2), async {
            while calls.load(Ordering::SeqCst) == 0 || state.sync_query.available_permits() != 1 {
                task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let gate = state.sync(|q| q.indexer().publication().clone());
        let writer = gate.clone();
        timeout(
            Duration::from_secs(2),
            task::spawn_blocking(move || writer.begin_update()),
        )
        .await
        .unwrap()
        .unwrap();
        gate.finish_update();
        let revision = gate.revision();
        assert!(!pending.is_finished());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        drop(held);
        let response = timeout(Duration::from_secs(2), pending)
            .await
            .unwrap()
            .unwrap();
        let bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(&bytes[..], revision.to_string().as_bytes());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn publication_wait_runs_once_and_retains_worker_admission() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    run(|state, _| async move {
        let gate = state.sync(|q| q.indexer().publication().clone());
        gate.begin_update();
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let reader = state.clone();
        let pending = spawn(async move {
            reader
                .read_admitted(move |q| {
                    count.fetch_add(1, Ordering::SeqCst);
                    q.resolve_blocks_v1(None, 10)
                        .and_then(|resolved| resolved.build(q))
                })
                .await
        });
        timeout(Duration::from_secs(2), async {
            while calls.load(Ordering::SeqCst) == 0 || state.sync_query.available_permits() != 0 {
                task::yield_now().await;
            }
        })
        .await
        .unwrap();
        time::sleep(Duration::from_millis(150)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!pending.is_finished());
        let next_state = state.clone();
        let mut next = spawn(async move { next_state.read_admitted(|_| Ok(42)).await });
        assert!(timeout(Duration::from_millis(50), &mut next).await.is_err());
        gate.finish_update();
        assert!(
            !timeout(Duration::from_secs(2), pending)
                .await
                .unwrap()
                .unwrap()
                .unwrap()
                .is_empty()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(next.await.unwrap().unwrap(), 42);
        assert_eq!(state.sync_query.available_permits(), 1);
    });
}

#[test]
fn generic_body_jobs_remain_admitted_after_cancellation() {
    run(|state, _| async move {
        for bound in [false, true] {
            let (started, ready) = oneshot::channel();
            let (release, blocked) = mpsc::channel();
            let first_state = state.clone();
            let first = spawn(async move {
                if bound {
                    let signals = Arc::new(Mutex::new(Some((started, blocked))));
                    first_state
                        .respond_json_bound(&HeaderMap::new(), Version::ONE, move |_| {
                            let (started, blocked) = signals.lock().unwrap().take().unwrap();
                            started.send(()).unwrap();
                            blocked.recv().unwrap();
                            Ok((b"[]".to_vec(), RepresentationId::content(b"[]")))
                        })
                        .await
                } else {
                    let signals = Arc::new(Mutex::new(Some((started, blocked))));
                    first_state
                        .respond_with_params(
                            &HeaderMap::new(),
                            CacheParams::resolve(
                                &CacheStrategy::Immutable(Version::ONE),
                                first_state.cdn_cache_mode,
                            ),
                            |_| {},
                            move |_| {
                                let (started, blocked) = signals.lock().unwrap().take().unwrap();
                                started.send(()).unwrap();
                                blocked.recv().unwrap();
                                Ok(Bytes::from_static(b"[]"))
                            },
                        )
                        .await
                }
            });
            timeout(Duration::from_secs(2), ready)
                .await
                .unwrap()
                .unwrap();
            first.abort();
            let _ = first.await;
            assert_eq!(state.sync_query.available_permits(), 0);

            // Already-known validators must not wait for a body permit.
            let mut headers = HeaderMap::new();
            headers.insert(IF_NONE_MATCH, "*".parse().unwrap());
            let cached = timeout(
                Duration::from_millis(100),
                state.respond_with_params(
                    &headers,
                    CacheParams::resolve(
                        &CacheStrategy::Immutable(Version::ONE),
                        state.cdn_cache_mode,
                    ),
                    |_| {},
                    |_| panic!("304 built a body"),
                ),
            )
            .await
            .unwrap();
            assert_eq!(cached.status(), StatusCode::NOT_MODIFIED);

            let next_state = state.clone();
            let mut next = spawn(async move {
                next_state
                    .respond_with_params(
                        &HeaderMap::new(),
                        CacheParams::deploy(),
                        |_| {},
                        |_| Ok(Bytes::from_static(b"[]")),
                    )
                    .await
            });
            assert!(timeout(Duration::from_millis(50), &mut next).await.is_err());
            release.send(()).unwrap();
            assert_eq!(
                timeout(Duration::from_secs(2), next)
                    .await
                    .unwrap()
                    .unwrap()
                    .status(),
                StatusCode::OK
            );
            assert_eq!(state.sync_query.available_permits(), 1);
        }
    });
}
