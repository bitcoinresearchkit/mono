use std::sync::atomic::Ordering;

use bitview_plugin::UpdateContext;
use bitview_runtime::update;
use brk_exit::Exit;
use brk_types::Timestamp;
use serde_json::{Value, from_str};

use super::{
    chain_fixture::{default_first, run_genesis},
    series_publication,
    server_routes::exchange_with_etag,
};

#[test]
fn date_and_timestamp_ranges_follow_reorganizations() {
    run_genesis(default_first(), |mut fixture| async move {
        fixture.publish(2, 1);
        let address = fixture.address;
        let chain = fixture.chain.clone();
        let active = fixture.active.clone();
        let tip = fixture.tip.clone();
        let plugins = &mut fixture.plugins;
        #[cfg(feature = "series")]
        {
            tip.store(2, Ordering::SeqCst);
            let boundary = (chain[1].header.time + chain[2].header.time) / 2;
            let path = format!(
                "/api/series/timestamp/height?start={}&limit=1",
                Timestamp::new(boundary).to_iso8601()
            );
            let mut timestamp_etag = "\"old\"".to_owned();
            for (branch, parent_time) in [(3, chain[2].header.time), (4, chain[1].header.time)] {
                let expected_start = if parent_time >= boundary { 1 } else { 2 };
                active.store(branch, Ordering::SeqCst);
                update(plugins, UpdateContext::new(&Exit::default())).unwrap();
                let response = exchange_with_etag(address, "GET", &path, &timestamp_etag).await;
                assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                assert_eq!(
                    body["start"], expected_start,
                    "timestamp range must follow the current fork: {body}"
                );
                timestamp_etag = response
                    .lines()
                    .find_map(|line| line.strip_prefix("etag: "))
                    .unwrap()
                    .to_owned();
                for method in ["GET", "HEAD"] {
                    let response =
                        exchange_with_etag(address, method, &path, &timestamp_etag).await;
                    assert!(response.starts_with("HTTP/1.1 304"), "{response}");
                    assert!(response.ends_with("\r\n\r\n"));
                }
            }
        }
    });
}

#[test]
fn daily_bucket_publication_remains_coherent() {
    run_genesis(default_first(), |mut fixture| async move {
        fixture.publish(6, 4);
        series_publication::check(
            &fixture.state,
            fixture.address,
            fixture.chain[5].header.time,
        )
        .await;
    });
}
