#[cfg(feature = "chain")]
mod addr_publication;
#[cfg(feature = "chain")]
mod admission;
#[cfg(feature = "chain")]
mod broadcast;
#[cfg(all(feature = "chain", feature = "series"))]
mod cache_reorg;
#[cfg(feature = "chain")]
mod cdn_mode;
#[cfg(feature = "chain")]
mod chain_fixture;
#[cfg(any(feature = "chain", feature = "price"))]
mod chain_rpc;
#[cfg(feature = "chain")]
mod cumulative_sources;
#[cfg(feature = "chain")]
mod genesis_routes;
#[cfg(feature = "chain")]
mod header_integrity;
#[cfg(feature = "chain")]
mod historical_price;
#[cfg(feature = "chain")]
mod mempool;
#[cfg(feature = "chain")]
mod mempool_publication;
mod middleware;
#[cfg(feature = "chain")]
mod mining;
#[cfg(feature = "price")]
mod oracle;
#[cfg(feature = "chain")]
mod raw_responses;
#[cfg(all(feature = "chain", feature = "series"))]
mod resolution_mappings;
#[cfg(feature = "chain")]
mod safe_prefix;
#[cfg(feature = "series")]
mod series_admission;
#[cfg(feature = "series")]
mod series_publication;
#[cfg(all(feature = "chain", feature = "series"))]
mod series_ranges;
mod server_routes;
#[cfg(feature = "chain")]
mod sync_success;
#[cfg(feature = "chain")]
mod transaction_publication;
#[cfg(feature = "urpd")]
mod urpd;
#[cfg(feature = "urpd")]
mod urpd_sources;

use super::json_error::is_json_content_type;

#[test]
fn json_content_type_matches() {
    assert!(is_json_content_type("application/json"));
    assert!(is_json_content_type("application/json; charset=utf-8"));
    assert!(is_json_content_type("  application/json  "));
    assert!(is_json_content_type("application/problem+json"));
    assert!(is_json_content_type(
        "application/vnd.api+json; charset=utf-8"
    ));
}

#[test]
fn json_content_type_rejects_non_json() {
    assert!(!is_json_content_type("text/plain"));
    assert!(!is_json_content_type("application/xml"));
    assert!(!is_json_content_type("application/json+xml"));
    assert!(!is_json_content_type(""));
    assert!(!is_json_content_type("text/json"));
}
