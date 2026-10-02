use axum::{http::header::ETAG, response::Response};
use bitcoin::{BlockHash as BitcoinBlockHash, hashes::Hash};
use brk_types::{BlockHash, Dollars};

use super::blocks_v1_params;
use crate::extended::ResponseExtended;

#[test]
fn v1_identity_covers_full_tip_and_ordered_body_prices() {
    let tip = BlockHash::default();
    let prices = [Dollars::from(10.0), Dollars::from(20.0)];
    let tag = |tip, prices: &[Dollars]| {
        Response::new_not_modified(&blocks_v1_params(tip, prices)).headers()[ETAG].clone()
    };
    let expected = tag(Some(tip), &prices);
    assert!(expected.to_str().unwrap().starts_with("W/\"blocks-v1-4-"));
    assert_eq!(expected, tag(Some(tip), &prices));
    assert_ne!(expected, tag(Some(tip), &[prices[1], prices[0]]));
    assert_ne!(expected, tag(Some(tip), &[prices[0], Dollars::from(20.01)]));
    assert_ne!(expected, tag(Some(tip), &prices[..1]));
    let mut changed = [0; 32];
    changed[31] = 1;
    assert_ne!(
        expected,
        tag(
            Some(BitcoinBlockHash::from_byte_array(changed).into()),
            &prices
        )
    );
    assert_ne!(tag(None, &[]), tag(Some(tip), &[]));
}
