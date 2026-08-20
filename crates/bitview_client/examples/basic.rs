//! Basic example of using the Bitview client.

use bitview_client::FormatResponse;
use bitview_client::{BitviewClient, BitviewClientOptions};
use brk_types::{Index, RangeIndex};

fn main() -> bitview_client::Result<()> {
    // Create client with default options
    let client = BitviewClient::new("http://localhost:3110");

    // Or with custom options
    let _client_with_options = BitviewClient::with_options(BitviewClientOptions {
        base_url: "http://localhost:3110".to_string(),
        timeout_secs: 60,
    });

    // Fetch price data using the typed series API.
    // day1() returns DateSeriesEndpoint, so fetch() returns DateSeriesData.
    let price_close = client
        .series()
        .price
        .split
        .close
        .usd
        .by
        .day1()
        .last(3)
        .fetch()?;
    println!("Last 3 price close values:");
    // iter_dates() returns Option (None for sub-daily indexes)
    for (date, value) in price_close.iter_dates().unwrap() {
        println!("  {}: {}", date, value);
    }
    // iter_timestamps() works for all date-based indexes including sub-daily
    for (ts, value) in price_close.iter_timestamps() {
        println!("  {}: {}", ts, value);
    }

    // Fetch block data through another date-based series.
    let block_count = client
        .series()
        .blocks
        .count
        .total
        .sum
        ._24h
        .by
        .day1()
        .last(3)
        .fetch()?;
    println!("Last 3 block count values:");
    for (date, value) in block_count.iter_dates().unwrap() {
        println!("  {}: {}", date, value);
    }

    // Fetch supply data as CSV
    dbg!(client.series().supply.circulating.btc.by.day1().path());
    let circulating = client
        .series()
        .supply
        .circulating
        .btc
        .by
        .day1()
        .last(3)
        .fetch_csv()?;
    println!("Last 3 circulating supply (CSV): {:?}", circulating);

    // Use a dynamic date-series endpoint when the name is known only at runtime.
    let date_series = client
        .date_series_endpoint("price_close", Index::Day1)?
        .last(3)
        .fetch()?;
    println!("Dynamic date series fetch:");
    for (date, value) in date_series.iter_dates().unwrap() {
        println!("  {}: {}", date, value);
    }

    // Use the generated REST method when the response format is selected dynamically.
    let series_data = client.get_series(
        "price_close".into(),
        Index::Day1,
        Some(RangeIndex::Int(-3)),
        None,
        None,
        None,
    )?;
    match series_data {
        FormatResponse::Json(m) => {
            println!("Generic fetch result count: {}", m.data.len());
        }
        FormatResponse::Csv(_) => panic!(),
    };

    Ok(())
}
