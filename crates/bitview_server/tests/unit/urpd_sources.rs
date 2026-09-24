use std::{collections::BTreeMap, fs, io::ErrorKind, net::SocketAddr, path::PathBuf};

use bitview_cohort::UTXOAggregateId;
use bitview_plugin_distribution::UTXOStates;
use bitview_urpd::{AgeRangeUrpds, UrpdRaw, accumulate_masses};
use brk_types::{
    Cents, CentsCompact, Cohort, Date, Sats, SupplyState, UrpdAggregation, UrpdWeight,
};
use serde_json::{Value, from_str, to_value};
use tempfile::tempdir;

use super::server_routes::exchange_with_etag;
use crate::AppState;

/// Exercise production packed writing and native/HTTP reads on a populated day.
pub async fn check(state: &AppState, address: SocketAddr) {
    let date = Date::new(2009, 1, 3);
    let path = state.sync(|q| q.plugins().distribution.states_path.clone());
    let staging = tempdir().unwrap();
    let mut producer = UTXOStates::new(staging.path());
    for age in producer.age_range.iter_mut() {
        age.reset_cost_basis_data_if_needed().unwrap();
    }
    for (price, sats) in [(101, 100_000_000_u64), (102, 200_000_000)] {
        producer.age_range.iter_mut().next().unwrap().receive_utxo(
            &SupplyState {
                utxo_count: 1,
                value: Sats::from(sats),
            },
            Cents::new(price),
        );
    }
    producer.age_range.iter_mut().last().unwrap().receive_utxo(
        &SupplyState {
            utxo_count: 1,
            value: Sats::from(500_000_000_u64),
        },
        Cents::new(200),
    );
    for age in producer.age_range.iter_mut() {
        age.apply_pending();
    }
    let mut originals = Vec::new();
    remember(&mut originals, AgeRangeUrpds::path(&path, date));
    producer.write_urpds(date, &path).unwrap();

    // Live and persisted entries share the same rounded price buckets.
    let buckets = |current| {
        producer
            .with_urpd_entries(&path, date, current, |entries| {
                accumulate_masses(entries, |total: &mut Sats, _, _, sats| *total += sats)
            })
            .unwrap()
            .unwrap()
    };
    assert_eq!(buckets(true), buckets(false));
    assert_eq!(
        buckets(false),
        BTreeMap::from([
            (CentsCompact::new(100), Sats::from(300_000_000_u64)),
            (CentsCompact::new(200), Sats::from(500_000_000_u64)),
        ])
    );

    for (name, total) in [
        ("all", 8.0),
        ("sth", 3.0),
        ("lth", 5.0),
        ("utxos_under_1h_old", 3.0),
    ] {
        let cohort = Cohort::new(name).unwrap();
        for aggregation in [
            UrpdAggregation::Raw,
            UrpdAggregation::Lin200,
            UrpdAggregation::Log100,
        ] {
            check_representation(
                state,
                address,
                &cohort,
                date,
                aggregation,
                UrpdWeight::Raw,
                total,
            )
            .await;
        }
    }

    // Weighted aggregate files are independent persisted sources. Use the
    // existing raw serializer, with different totals to detect wrong selection.
    let cohort = Cohort::new("all").unwrap();
    for (weight, sats) in [
        (UrpdWeight::Cointime, 200_000_000_u64),
        (UrpdWeight::Coinflow, 400_000_000),
    ] {
        let dir = state.sync(|q| match weight {
            UrpdWeight::Cointime => q.plugins().cointime.urpd.dir(UTXOAggregateId::All),
            UrpdWeight::Coinflow => q.plugins().coinflow.urpd.dir(UTXOAggregateId::All),
            UrpdWeight::Raw => unreachable!(),
        });
        let file = dir.join(date.to_string());
        remember(&mut originals, file.clone());
        fs::create_dir_all(dir).unwrap();
        let raw = UrpdRaw {
            map: BTreeMap::from([(CentsCompact::new(100), Sats::from(sats))]),
        };
        fs::write(file, raw.serialize().unwrap()).unwrap();
        for aggregation in [UrpdAggregation::Raw, UrpdAggregation::Lin200] {
            check_representation(
                state,
                address,
                &cohort,
                date,
                aggregation,
                weight,
                sats as f64 / 100_000_000.0,
            )
            .await;
        }
    }
    // Restore only these fixture-owned files before the reorg checks continue.
    for (file, previous) in originals {
        match previous {
            Some(bytes) => fs::write(file, bytes).unwrap(),
            None => fs::remove_file(file).unwrap(),
        }
    }
}

fn remember(originals: &mut Vec<(PathBuf, Option<Vec<u8>>)>, path: PathBuf) {
    let bytes = match fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => panic!("cannot capture fixture snapshot: {error}"),
    };
    originals.push((path, bytes));
}

async fn check_representation(
    state: &AppState,
    address: SocketAddr,
    cohort: &Cohort,
    date: Date,
    aggregation: UrpdAggregation,
    weight: UrpdWeight,
    total: f64,
) {
    let expected = state
        .sync(|q| {
            q.resolve_urpd_at(cohort, date, aggregation, weight)
                .and_then(|resolved| resolved.build())
        })
        .unwrap();
    let expected = to_value(expected).unwrap();
    assert_eq!(expected["total_supply"], total);
    let mut previous_tag = None;
    for suffix in [String::new(), format!("/{date}")] {
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
