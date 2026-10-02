use bitcoin::Amount;
use bitview_plugin::{ComputePlugin, ImportContext, UpdateContext};
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_indexer::HasIndexer;
use bitview_plugin_inputs::HasInputs;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_mining::HasMining;
use bitview_plugin_price::HasPrice;
use bitview_plugin_transactions::{Dependencies, HasTransactions, Vecs as Transactions};
use brk_exit::Exit;
use brk_types::{Sats, Version};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, ReadableVec, WritableVec};

use super::chain_fixture::{ChainFixture, raw_fixture_block, run_genesis};

fn compute(transactions: &mut Transactions, fixture: &ChainFixture) {
    transactions
        .compute(
            Dependencies {
                indexer: fixture.plugins.indexer(),
                inputs: fixture.plugins.inputs(),
                mappings: fixture.plugins.mappings(),
                blocks: fixture.plugins.blocks(),
                price: fixture.plugins.price(),
            },
            UpdateContext::new(&Exit::default()),
        )
        .unwrap();
}

fn check(transactions: &Transactions) {
    assert_eq!(
        transactions.fees.coinbase_value.collect(),
        [Sats::FIFTY_BTC, Sats::ONE_BTC * 52usize]
    );
    assert_eq!(
        transactions.fees.fee.tx_index.collect(),
        [Sats::ZERO, Sats::ZERO, Sats::ONE_BTC, Sats::ONE_BTC]
    );
    assert_eq!(
        transactions.fees.total.collect(),
        [Sats::ZERO, Sats::ONE_BTC * 2usize]
    );
    assert_eq!(
        transactions
            .volume
            .transfer_volume
            .cumulative
            .sats
            .height
            .collect(),
        [Sats::ZERO, Sats::ONE_BTC * 19usize]
    );
}

#[test]
fn transaction_values_feed_mining_and_survive_reopen_and_reorgs() {
    let mut first = raw_fixture_block();
    first.txdata[0].output[0].value = Amount::from_sat(4_200_000_000);
    first.txdata[0].output[2].value = Amount::from_sat(1_000_000_000);
    let coinbase = first.txdata[0].compute_txid();
    for input in &mut first.txdata[1].input {
        input.previous_output.txid = coinbase;
    }
    first.txdata[1].output[0].value = Amount::from_sat(900_000_000);
    first.txdata[2].input[0].previous_output.txid = first.txdata[1].compute_txid();
    first.txdata[2].output[0].value = Amount::from_sat(800_000_000);
    first.header.merkle_root = first.compute_merkle_root().unwrap();

    run_genesis(first, |mut fixture| async move {
        fixture.publish(1, 1);
        check(fixture.plugins.transactions());
        let rewards = &fixture.plugins.mining().rewards;
        assert_eq!(
            rewards.coinbase.block.sats.collect(),
            [Sats::FIFTY_BTC, Sats::ONE_BTC * 52usize]
        );
        assert_eq!(
            rewards.fees.block.sats.collect(),
            [Sats::ZERO, Sats::ONE_BTC * 2usize]
        );
        assert_eq!(rewards.subsidy.block.sats.collect(), [Sats::FIFTY_BTC; 2]);
        assert_eq!(
            rewards.output_volume.collect(),
            [Sats::ZERO, Sats::ONE_BTC * 17usize]
        );

        // A separate writer exercises schema revisions and reopening without
        // modifying the published snapshot, using fully computed dependencies.
        let directory = tempdir().unwrap();
        let import = || {
            Transactions::import(
                ImportContext::new(directory.path()),
                fixture.plugins.indexer(),
                fixture.plugins.mappings(),
                &fixture.plugins.blocks().lookback.window_starts(),
            )
            .unwrap()
        };
        let mut transactions = import();
        compute(&mut transactions, &fixture);
        check(&transactions);
        let rates = transactions.fees.fee_rate.collect();
        let effective = transactions.fees.effective_fee_rate.tx_index.collect();
        transactions
            .fees
            .fee
            .tx_index
            .validate_computed_version_or_reset(Version::ZERO)
            .unwrap();
        compute(&mut transactions, &fixture);
        check(&transactions);
        transactions
            .fees
            .coinbase_value
            .validate_computed_version_or_reset(Version::ZERO)
            .unwrap();
        compute(&mut transactions, &fixture);
        check(&transactions);

        // Reopen with a newly added total and revised rates. Rebuilding them
        // from the saved fee prefix must preserve monetary and CPFP results.
        transactions.fees.total.truncate_if_needed_at(0).unwrap();
        transactions.fees.total.write().unwrap();
        transactions
            .fees
            .fee_rate
            .validate_computed_version_or_reset(Version::ZERO)
            .unwrap();
        transactions.fees.fee_rate.write().unwrap();
        transactions
            .fees
            .effective_fee_rate
            .tx_index
            .validate_computed_version_or_reset(Version::ZERO)
            .unwrap();
        transactions
            .fees
            .effective_fee_rate
            .tx_index
            .write()
            .unwrap();
        drop(transactions);
        let mut reopened = import();
        compute(&mut reopened, &fixture);
        check(&reopened);
        assert_eq!(reopened.fees.fee_rate.collect(), rates);
        assert_eq!(
            reopened.fees.effective_fee_rate.tx_index.collect(),
            effective
        );
        drop(reopened);

        fixture.publish(2, 1);
        let transactions = fixture.plugins.transactions();
        assert_eq!(transactions.fees.fee.tx_index.collect(), [Sats::ZERO; 2]);
        assert_eq!(
            transactions.fees.coinbase_value.collect(),
            [Sats::FIFTY_BTC; 2]
        );
        assert_eq!(
            transactions
                .volume
                .transfer_volume
                .cumulative
                .sats
                .height
                .collect(),
            [Sats::ZERO; 2]
        );
        assert_eq!(
            fixture
                .plugins
                .mining()
                .rewards
                .coinbase
                .block
                .sats
                .collect(),
            [Sats::FIFTY_BTC; 2]
        );
        assert_eq!(
            fixture.plugins.mining().rewards.fees.block.sats.collect(),
            [Sats::ZERO; 2]
        );
        fixture.publish(1, 1);
        check(fixture.plugins.transactions());
    });
}
