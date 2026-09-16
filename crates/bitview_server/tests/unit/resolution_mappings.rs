use std::net::SocketAddr;

use brk_types::{
    Date, Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, Timestamp, Week1, Year1, Year10,
};
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
            // Public reverse mappings and plugin-facing views must agree with
            // the timestamp/height definitions after every publication.
            for (metric, periods) in [
                (
                    "minute10",
                    timestamps
                        .iter()
                        .copied()
                        .map(Minute10::from_timestamp)
                        .map(usize::from)
                        .collect::<Vec<_>>(),
                ),
                (
                    "minute30",
                    timestamps
                        .iter()
                        .copied()
                        .map(Minute30::from_timestamp)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "hour1",
                    timestamps
                        .iter()
                        .copied()
                        .map(Hour1::from_timestamp)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "hour4",
                    timestamps
                        .iter()
                        .copied()
                        .map(Hour4::from_timestamp)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "hour12",
                    timestamps
                        .iter()
                        .copied()
                        .map(Hour12::from_timestamp)
                        .map(usize::from)
                        .collect(),
                ),
                ("day1", days.iter().copied().map(usize::from).collect()),
                (
                    "day3",
                    timestamps
                        .iter()
                        .copied()
                        .map(Day3::from_timestamp)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "week1",
                    days.iter()
                        .copied()
                        .map(Week1::from)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "month1",
                    days.iter()
                        .copied()
                        .map(Month1::from)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "month3",
                    days.iter()
                        .copied()
                        .map(Month1::from)
                        .map(Month3::from)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "month6",
                    days.iter()
                        .copied()
                        .map(Month1::from)
                        .map(Month6::from)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "year1",
                    days.iter()
                        .copied()
                        .map(Month1::from)
                        .map(Year1::from)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "year10",
                    days.iter()
                        .copied()
                        .map(Month1::from)
                        .map(Year1::from)
                        .map(Year10::from)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "epoch",
                    (0..timestamps.len())
                        .map(Height::from)
                        .map(Epoch::from)
                        .map(usize::from)
                        .collect(),
                ),
                (
                    "halving",
                    (0..timestamps.len())
                        .map(Height::from)
                        .map(Halving::from)
                        .map(usize::from)
                        .collect(),
                ),
            ] {
                assert_eq!(
                    data(fixture.address, metric, "height").await,
                    periods
                        .into_iter()
                        .map(|period| json!(period))
                        .collect::<Vec<_>>(),
                    "{metric}/height branch={branch}"
                );
            }
            for metric in [
                "coinflow_urpd_capitalized_price_cents",
                "awake_urpd_capitalized_price_cents",
                "bedrock_under_4m_cost_basis_min_cents",
                "bedrock_under_4m_cost_basis_max_cents",
                "bedrock_under_5m_cost_basis_min_cents",
                "bedrock_under_5m_cost_basis_max_cents",
                "bedrock_under_6m_cost_basis_min_cents",
                "bedrock_under_6m_cost_basis_max_cents",
            ] {
                let daily = data(fixture.address, metric, "day1").await;
                if metric.starts_with("bedrock_under_") {
                    // This pre-market fixture has occupied zero-price buckets,
                    // including on the current partial day after each reorg.
                    assert_eq!(daily.last(), Some(&json!(0)), "{metric} branch={branch}");
                }
                let expected: Vec<_> = days
                    .iter()
                    .map(|&day| daily.get(usize::from(day)).cloned().unwrap_or(Value::Null))
                    .collect();
                assert_eq!(
                    data(fixture.address, metric, "height").await,
                    expected,
                    "{metric}/height branch={branch}",
                );
            }
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

                for metric in [
                    "price_cents",
                    "supply_sats",
                    "coinflow_urpd_capitalized_price_ratio_ppm",
                    "awake_urpd_capitalized_price_ratio_ppm",
                    "coinflow_capitalized_price_cents",
                    "sth_coinflow_capitalized_price_cents",
                    "lth_coinflow_capitalized_price_cents",
                    "awake_capitalized_price_cents",
                    "sth_awake_capitalized_price_cents",
                    "lth_awake_capitalized_price_cents",
                    "over_4m_awake_price_cents",
                    "over_4m_awake_capitalized_price_cents",
                    "over_4m_coinflow_price_cents",
                    "over_4m_coinflow_capitalized_price_cents",
                    "over_6m_awake_price_cents",
                    "over_6m_awake_capitalized_price_cents",
                    "over_6m_coinflow_price_cents",
                    "over_6m_coinflow_capitalized_price_cents",
                    "under_4m_awake_price_cents",
                    "under_4m_awake_capitalized_price_cents",
                    "under_6m_awake_price_cents",
                    "under_6m_awake_capitalized_price_cents",
                    "under_4m_coinflow_price_cents",
                    "under_4m_coinflow_capitalized_price_cents",
                    "under_6m_coinflow_price_cents",
                    "under_6m_coinflow_capitalized_price_cents",
                    "coinflow_capitalized_price_ratio_ppm",
                    "awake_capitalized_price_ratio_ppm",
                ] {
                    let heights = data(fixture.address, metric, "height").await;
                    assert_eq!(heights.len(), height as usize + 1);
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
