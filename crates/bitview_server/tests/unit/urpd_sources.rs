use super::server_routes::exchange_with_etag;
use crate::AppState;
use brk_error::Error;
use brk_types::{Cohort, Date, UrpdAggregation, UrpdWeight};
use serde_json::{Value, from_str, to_value};
use std::net::SocketAddr;

/// Exercise history reconstruction through both native and HTTP queries.
pub async fn check(state: &AppState, address: SocketAddr) {
    let date = Date::new(2009, 1, 3);
    for name in [
        "all",
        "sth",
        "lth",
        "under_4m",
        "under_6m",
        "over_4m",
        "over_6m",
        "utxos_under_1h_old",
    ] {
        let cohort = Cohort::new(name).unwrap();
        for aggregation in [
            UrpdAggregation::Raw,
            UrpdAggregation::Lin200,
            UrpdAggregation::Log100,
        ] {
            for weight in [UrpdWeight::Raw, UrpdWeight::Cointime, UrpdWeight::Coinflow] {
                check_representation(state, address, &cohort, date, aggregation, weight).await;
            }
        }
    }
}

async fn check_representation(
    state: &AppState,
    address: SocketAddr,
    cohort: &Cohort,
    date: Date,
    aggregation: UrpdAggregation,
    weight: UrpdWeight,
) {
    let expected = state.sync(|q| {
        q.resolve_urpd_at(cohort, date, aggregation, weight)
            .and_then(|resolved| resolved.build())
    });
    let expected = match expected {
        Ok(value) => to_value(value).unwrap(),
        Err(Error::NotFound(_)) if weight != UrpdWeight::Raw => {
            // Early populated cohorts can lack a finite model weight. Neither a
            // conditional request nor a date alias may invent a distribution.
            for suffix in [String::new(), format!("/{date}"), "/1".to_owned()] {
                for method in ["GET", "HEAD"] {
                    let route =
                        format!("/api/urpd/{cohort}{suffix}?agg={aggregation}&weight={weight}");
                    let response = exchange_with_etag(address, method, &route, "*").await;
                    assert!(response.starts_with("HTTP/1.1 404"), "{response}");
                    assert!(!response.contains("\r\netag:"));
                }
            }
            return;
        }
        Err(error) => panic!("{cohort} {weight}: {error}"),
    };
    let height = expected["height"].as_u64().unwrap();
    let mut previous_tag = None;
    for suffix in [String::new(), format!("/{date}"), format!("/{height}")] {
        let route = format!("/api/urpd/{cohort}{suffix}?agg={aggregation}&weight={weight}");
        let response = exchange_with_etag(address, "GET", &route, "\"old\"").await;
        assert!(response.starts_with("HTTP/1.1 200"), "{route}: {response}");
        let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body, expected, "{route}");
        let tag = response
            .lines()
            .find_map(|line| line.strip_prefix("etag: "))
            .unwrap()
            .to_owned();
        if let Some(previous) = &previous_tag {
            assert_eq!(&tag, previous);
        }
        for method in ["GET", "HEAD"] {
            for condition in [&tag, "*"] {
                let response = exchange_with_etag(address, method, &route, condition).await;
                assert!(response.starts_with("HTTP/1.1 304"), "{route}: {response}");
                assert!(response.ends_with("\r\n\r\n"));
            }
        }
        previous_tag = Some(tag);
    }
}
