use std::net::SocketAddr;

use brk_types::{Date, Day1, Month1, Timestamp};
use serde_json::{Value, from_str, from_value, json};

use super::{
    chain_fixture::{default_first, run_genesis},
    server_routes::exchange_with_etag,
};

async fn data(address: SocketAddr, series: &str, index: &str) -> Vec<Value> {
    let response = exchange_with_etag(
        address,
        "GET",
        &format!("/api/series/{series}/{index}/data"),
        "\"old\"",
    )
    .await;
    assert!(
        response.starts_with("HTTP/1.1 200"),
        "{series}/{index}: {response}"
    );
    from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap()
}

#[test]
fn resident_resolution_mappings_preserve_last_values_through_append_and_reorg() {
    let mut first = default_first();
    first.header.time = *Timestamp::from(Date::new(2009, 2, 3));
    run_genesis(first, |mut fixture| async move {
        // Cross a month boundary, extend that period, then replace it with a
        // same-height branch in the previous month and grow that branch again.
        for (branch, height) in [(1, 1), (4, 2), (2, 1), (3, 2), (1, 1)] {
            fixture.publish(branch, height);
            let timestamps: Vec<Timestamp> = data(fixture.address, "timestamp_monotonic", "height")
                .await
                .into_iter()
                .map(|value| from_value(value).unwrap())
                .collect();
            let days: Vec<_> = timestamps
                .iter()
                .copied()
                .map(|timestamp| Day1::try_from(Date::from(timestamp)).unwrap())
                .collect();
            for (index, periods) in [
                (
                    "day1",
                    days.iter().copied().map(usize::from).collect::<Vec<_>>(),
                ),
                (
                    "month1",
                    days.iter()
                        .copied()
                        .map(Month1::from)
                        .map(usize::from)
                        .collect(),
                ),
            ] {
                let len = periods.last().unwrap() + 1;
                let firsts: Vec<_> = (0..len)
                    .map(|period| periods.partition_point(|&value| value < period))
                    .collect();
                assert_eq!(
                    data(fixture.address, "first_height", index).await,
                    firsts
                        .iter()
                        .map(|&height| json!(height))
                        .collect::<Vec<_>>(),
                );
                let dates: Vec<_> = firsts
                    .iter()
                    .enumerate()
                    .map(|(period, &first)| {
                        json!(if index == "day1" {
                            Date::from(Day1::from(period))
                        } else {
                            Date::from(timestamps[first])
                        })
                    })
                    .collect();
                assert_eq!(data(fixture.address, "date", index).await, dates);
                let expected_timestamps: Vec<_> = (0..len)
                    .map(|period| {
                        json!(if index == "day1" {
                            Day1::from(period).to_timestamp()
                        } else {
                            Month1::from(period).to_timestamp()
                        })
                    })
                    .collect();
                assert_eq!(
                    data(fixture.address, "timestamp", index).await,
                    expected_timestamps
                );

                // URPD values are block-based; each period selects its last block.
                for metric in [
                    "utxos_urpd_sth_cost_basis_min_cents",
                    "coinflow_urpd_all_capitalized_price_cents",
                    "cointime_urpd_all_capitalized_price_cents",
                    "cointime_urpd_all_capitalized_price_ratio_ppm",
                    "coinflow_capitalized_price_cents",
                    "awake_capitalized_price_cents",
                    "price_cents",
                    "supply_sats",
                ] {
                    let heights = data(fixture.address, metric, "height").await;
                    assert_eq!(heights.len(), height as usize + 1);
                    if metric.starts_with("utxos_urpd_") {
                        // Genesis has no spendable coins; later pre-market blocks
                        // have occupied zero-price buckets, including after a reorg.
                        assert_eq!(heights.first(), Some(&Value::Null), "{metric}");
                        assert_eq!(heights.last(), Some(&json!(0)), "{metric} branch={branch}");
                    }
                    let expected: Vec<_> = firsts
                        .iter()
                        .enumerate()
                        .map(|(period, &first)| {
                            let end = firsts
                                .get(period + 1)
                                .copied()
                                .unwrap_or(heights.len())
                                .min(heights.len());
                            if first < end {
                                heights[end - 1].clone()
                            } else {
                                Value::Null
                            }
                        })
                        .collect();
                    assert_eq!(
                        data(fixture.address, metric, index).await,
                        expected,
                        "{metric}/{index} branch={branch}"
                    );
                }
            }
        }
    });
}
