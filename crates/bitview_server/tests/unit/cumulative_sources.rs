use std::iter;

use bitview_plugin_indexer::HasIndexer;
use bitview_plugin_inputs::HasInputs;
use bitview_plugin_market::HasMarket;
use bitview_plugin_outputs::HasOutputs;
use bitview_plugin_price::HasPrice;
use brk_types::{OutputType, StoredU64};
use serde_json::{Value, from_str, to_value};
use vecdb::{AnyVec, ReadableVec, VecIndex};

use super::chain_fixture::{raw_fixture_block, run_genesis};
use super::server_routes::exchange_with_etag;

fn assert_counts(stored: Vec<StoredU64>, blocks: Vec<StoredU64>, expected: Vec<u64>) {
    let mut sum = 0;
    let cumulative: Vec<_> = expected
        .iter()
        .map(|&count| {
            sum += count;
            StoredU64::from(sum)
        })
        .collect();
    assert_eq!(stored, cumulative);
    assert_eq!(
        blocks.into_iter().map(u64::from).collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn cumulative_sources_follow_real_plugin_reorgs() {
    run_genesis(raw_fixture_block(), |mut fixture| async move {
        // Includes a same-height replacement, then an append and a rewind.
        for (branch, height) in [(1, 1), (2, 1), (1, 1), (4, 2), (2, 1)] {
            fixture.publish(branch, height);
            let indexer = fixture.plugins.indexer().vecs();
            let transactions = &indexer.transactions;
            let first = transactions.first_tx_index.collect();
            let mut block_inputs = Vec::new();
            let mut block_outputs = Vec::new();
            for (height, &from) in first.iter().enumerate() {
                let from = from.to_usize();
                let to = first
                    .get(height + 1)
                    .map(|i| i.to_usize())
                    .unwrap_or_else(|| transactions.txid.len());
                let mut inputs = Vec::new();
                let mut outputs = Vec::new();
                for tx in from..to {
                    let out_from = transactions
                        .first_txout_index
                        .collect_one_at(tx)
                        .unwrap()
                        .to_usize();
                    let out_to = transactions
                        .first_txout_index
                        .collect_one_at(tx + 1)
                        .map(|i| i.to_usize())
                        .unwrap_or_else(|| indexer.outputs.output_type.len());
                    indexer
                        .outputs
                        .output_type
                        .read_into_at(out_from, out_to, &mut outputs);
                    if tx != from {
                        let in_from = transactions
                            .first_txin_index
                            .collect_one_at(tx)
                            .unwrap()
                            .to_usize();
                        let in_to = transactions
                            .first_txin_index
                            .collect_one_at(tx + 1)
                            .map(|i| i.to_usize())
                            .unwrap_or_else(|| indexer.inputs.output_type.len());
                        indexer
                            .inputs
                            .output_type
                            .read_into_at(in_from, in_to, &mut inputs);
                    }
                }
                block_inputs.push(inputs);
                block_outputs.push(outputs);
            }
            let inputs = &fixture.plugins.inputs().by_type;
            let outputs = &fixture.plugins.outputs().by_type;
            for (kind, stored) in inputs.input_count_stored.iter_typed() {
                let expected: Vec<_> = block_inputs
                    .iter()
                    .map(|values| values.iter().filter(|&&value| value == kind).count() as u64)
                    .collect();
                assert_counts(
                    stored.collect(),
                    inputs.input_count.by_type.get(kind).block.collect(),
                    expected,
                );
            }
            for (kind, stored) in
                outputs
                    .output_count_stored
                    .spendable
                    .iter_typed()
                    .chain(iter::once((
                        OutputType::OpReturn,
                        &outputs.output_count_stored.unspendable.op_return,
                    )))
            {
                let expected: Vec<_> = block_outputs
                    .iter()
                    .map(|values| values.iter().filter(|&&value| value == kind).count() as u64)
                    .collect();
                assert_counts(
                    stored.collect(),
                    outputs.output_count.by_type.get(kind).block.collect(),
                    expected,
                );
            }
            let mut sum = 0;
            let expected: Vec<_> = fixture
                .plugins
                .price()
                .spot
                .cents
                .height
                .collect()
                .iter()
                .map(|p| {
                    sum += p.inner();
                    sum
                })
                .collect();
            assert_eq!(
                fixture
                    .plugins
                    .market()
                    .moving_average
                    .sma_prefix_sum
                    .collect()
                    .into_iter()
                    .map(u64::from)
                    .collect::<Vec<_>>(),
                expected
            );

            let price = fixture.plugins.price();
            let expected_open: Vec<_> = price
                .ohlc
                .usd
                .day1
                .collect()
                .into_iter()
                .map(|candle| *candle.open)
                .collect();
            assert_eq!(price.split.open.usd.day1.collect(), expected_open);
            let response = exchange_with_etag(
                fixture.address,
                "GET",
                "/api/series/price_open/day1/data",
                "\"old\"",
            )
            .await;
            assert!(response.starts_with("HTTP/1.1 200"), "{response}");
            assert_eq!(
                from_str::<Value>(response.split_once("\r\n\r\n").unwrap().1).unwrap(),
                to_value(expected_open).unwrap(),
            );
        }
    });
}
