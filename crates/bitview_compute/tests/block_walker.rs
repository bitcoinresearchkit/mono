use std::iter;

use bitview_compute::{BlockAggregate, CoinbasePolicy, walk_blocks};
use brk_types::{OutputType, TxIndex};

fn scan(coinbase: CoinbasePolicy) -> Vec<BlockAggregate> {
    // Two blocks, the second containing only its coinbase. Start mid-chain.
    let types = [0, 0, 1, 1, 2, 1, 0, 2];
    let mut remaining = types.as_slice();
    let mut blocks = Vec::new();
    walk_blocks(
        &[TxIndex::new(10), TxIndex::new(13)],
        14,
        100..108,
        [102, 105, 106].into_iter(),
        coinbase,
        |count, target| {
            let (entries, rest) = remaining.split_at(count);
            if let Some(target) = target {
                for &kind in entries {
                    target[kind] += 1;
                }
            }
            remaining = rest;
        },
        |block| {
            blocks.push(block);
            Ok(())
        },
    )
    .unwrap();
    assert!(remaining.is_empty());
    blocks
}

#[test]
fn outputs_count_entries_and_each_transaction_once_per_type() {
    let blocks = scan(CoinbasePolicy::Include);
    assert_eq!(blocks.len(), 2);
    assert_eq!(&blocks[0].entries_per_type[..3], &[2, 3, 1]);
    assert_eq!(&blocks[0].txs_per_type[..3], &[1, 2, 1]);
    assert_eq!(&blocks[1].entries_per_type[..3], &[1, 0, 1]);
    assert_eq!(&blocks[1].txs_per_type[..3], &[1, 0, 1]);
    for block in blocks {
        assert!(block.entries_per_type[3..].iter().all(|&n| n == 0));
        assert!(block.txs_per_type[3..].iter().all(|&n| n == 0));
    }
}

#[test]
fn inputs_skip_coinbase_entries_including_a_final_coinbase_only_block() {
    let blocks = scan(CoinbasePolicy::Skip);
    assert_eq!(&blocks[0].entries_per_type[..3], &[0, 3, 1]);
    assert_eq!(&blocks[0].txs_per_type[..3], &[0, 2, 1]);
    assert_eq!(blocks[1].entries_per_type, [0; OutputType::COUNT]);
    assert_eq!(blocks[1].txs_per_type, [0; OutputType::COUNT]);
}

#[test]
fn an_empty_block_range_does_not_read_or_store() {
    for coinbase in [CoinbasePolicy::Include, CoinbasePolicy::Skip] {
        walk_blocks(
            &[],
            10,
            100..100,
            iter::from_fn(|| panic!("unexpected source read")),
            coinbase,
            |_, _| panic!("unexpected scan"),
            |_| panic!("unexpected store"),
        )
        .unwrap();
    }
}

#[test]
fn a_missing_transaction_boundary_fails_without_storing_the_block() {
    assert!(
        walk_blocks(
            &[TxIndex::new(10)],
            12,
            100..102,
            iter::empty(),
            CoinbasePolicy::Include,
            |_, _| {},
            |_| panic!("incomplete block must not be stored"),
        )
        .is_err()
    );
}
