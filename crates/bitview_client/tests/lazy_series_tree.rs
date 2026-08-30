use bitview_client::{AnySeriesPattern, BitviewClient, Dollars, SeriesPattern2, SeriesTree_Price};

fn pattern_name(pattern: &impl AnySeriesPattern) -> &str {
    pattern.name()
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn typed_series_path_initializes_on_the_default_stack() {
    assert_send_sync::<BitviewClient>();
    let client = BitviewClient::new("http://localhost:3110");

    let price: &SeriesTree_Price = &client.series().price;
    assert!(std::ptr::eq(price, &**client.series().price));
    let usd: &SeriesPattern2<Dollars> = &price.split.close.usd;

    assert_eq!(pattern_name(&price.split.close.usd), "price_close");
    assert_eq!(usd.name(), "price_close");
    assert_eq!(
        client.series().cohorts.supply.total.all.btc.name(),
        "supply"
    );
}
