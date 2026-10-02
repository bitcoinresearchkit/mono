use bitview_plugin_distribution_aggregated::{
    Dependencies as AggregatedDependencies, HasDistributionAggregated, Vecs as Aggregated,
};
use std::{collections::BTreeMap, path::Path};

use bitcoin::Amount;
use bitview_plugin::{ComputePlugin, ImportContext, PluginData, UpdateContext};
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_distribution_addresses::{
    Dependencies as AddressesDependencies, HasDistributionAddresses, Vecs as Addresses,
};
use bitview_plugin_distribution_age::{
    Dependencies as AgeDependencies, HasDistributionAge, Vecs as Age,
};
use bitview_plugin_distribution_utxos::{
    Dependencies as UtxosDependencies, HasDistributionUtxos, Vecs as Utxos,
};
use bitview_plugin_indexer::HasIndexer;
use bitview_plugin_inputs::HasInputs;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_outputs::HasOutputs;
use bitview_plugin_price::HasPrice;
use bitview_plugin_utxo_history::HasUtxoHistory;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use tempfile::tempdir;
use vecdb::{AnyVec, ReadableCloneableVec, ReadableVec};

use super::chain_fixture::{ChainFixture, raw_fixture_block, run_genesis};

type Distribution = (Age, Utxos, Addresses, Aggregated);

fn import(path: &Path, fixture: &ChainFixture) -> Distribution {
    let plugins = &fixture.plugins;
    let context = ImportContext::new(path);
    let windows = plugins.blocks().lookback.window_starts();
    let supply = plugins.utxo_history().supply.read_only_boxed_clone();
    (
        Age::import(
            context,
            plugins.mappings(),
            &windows,
            plugins.price(),
            &supply,
        )
        .unwrap(),
        Utxos::import(
            context,
            plugins.mappings(),
            &windows,
            plugins.price(),
            &supply,
        )
        .unwrap(),
        Addresses::import(
            context,
            plugins.mappings(),
            &windows,
            plugins.price(),
            &plugins.inputs().by_type,
            &plugins.outputs().by_type,
            &supply,
        )
        .unwrap(),
        Aggregated::import(
            context,
            plugins.mappings(),
            &windows,
            plugins.price(),
            &supply,
        )
        .unwrap(),
    )
}

fn compute(writer: &mut Distribution, fixture: &ChainFixture) -> Result<()> {
    let plugins = &fixture.plugins;
    let history = plugins.utxo_history().reader(
        plugins.inputs().origins.spends(),
        &plugins.outputs().creations,
    )?;
    writer.0.compute(
        AgeDependencies {
            history: &history,
            from: plugins.indexer().safe_lengths().height,
            mappings: plugins.mappings(),
            price: plugins.price(),
        },
        UpdateContext::new(&Exit::default()),
    )?;
    writer.3.compute(
        AggregatedDependencies {
            history: &history,
            from: plugins.indexer().safe_lengths().height,
            age: &writer.0,
            mappings: plugins.mappings(),
            price: plugins.price(),
        },
        UpdateContext::new(&Exit::default()),
    )?;
    writer.1.compute(
        UtxosDependencies {
            indexer: plugins.indexer(),
            mappings: plugins.mappings(),
            input_values: &plugins.inputs().value,
            price: plugins.price(),
        },
        UpdateContext::new(&Exit::default()),
    )?;
    let history = plugins.utxo_history();
    for h in 0..history.count.height.len() {
        let height = Height::from(h);
        let supply = history.supply.collect_one(height).unwrap();
        let count = history.count.height.collect_one(height).unwrap();
        assert_eq!(
            writer
                .1
                .cohorts
                .outputs
                .avg_amount
                .all
                .sats
                .height
                .collect_one(height),
            Some(supply / count),
            "average UTXO amount at {h}",
        );
    }
    writer.2.compute(
        AddressesDependencies {
            indexer: plugins.indexer(),
            mappings: plugins.mappings(),
            input_values: &plugins.inputs().value,
            price: plugins.price(),
            type_supply: writer.1.type_supply(),
        },
        UpdateContext::new(&Exit::default()),
    )
}

fn series(writer: &impl PluginData) -> BTreeMap<String, Vec<u8>> {
    let mut result = BTreeMap::new();
    writer.for_each_visible(&mut |vec| {
        let mut values = Vec::new();
        vec.write_json(None, None, &mut values).unwrap();
        result.insert(vec.region_name(), values);
    });
    result
}

fn compare(writer: &impl PluginData, fresh: &impl PluginData) {
    let expected = series(fresh);
    for (name, values) in series(writer) {
        assert_eq!(values, expected[&name], "{name}");
    }
}

fn check(writer: &mut Distribution, fixture: &ChainFixture) {
    compute(writer, fixture).unwrap();
    let directory = tempdir().unwrap();
    let mut fresh = import(directory.path(), fixture);
    compute(&mut fresh, fixture).unwrap();
    compare(&writer.0, &fresh.0);
    compare(&writer.1, &fresh.1);
    compare(&writer.2, &fresh.2);
    compare(&writer.3, &fresh.3);
}

fn check_published(fixture: &ChainFixture) {
    let directory = tempdir().unwrap();
    let mut fresh = import(directory.path(), fixture);
    compute(&mut fresh, fixture).unwrap();
    compare(fixture.plugins.distribution_age(), &fresh.0);
    compare(fixture.plugins.distribution_utxos(), &fresh.1);
    compare(fixture.plugins.distribution_addresses(), &fresh.2);
    compare(fixture.plugins.distribution_aggregated(), &fresh.3);
}

#[test]
fn distribution_live_state_matches_rebuild_after_append_reopen_and_reorg() {
    let mut first = raw_fixture_block();
    first.txdata[0].output[0].value = Amount::from_sat(4_000_000_000);
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
        fixture.publish(1, 0);
        let directory = tempdir().unwrap();
        let mut writer = import(directory.path(), &fixture);
        check(&mut writer, &fixture);

        for (branch, height) in [(1, 1), (4, 2), (5, 3), (6, 4)] {
            fixture.publish(branch, height);
            check(&mut writer, &fixture);
        }
        drop(writer);
        let mut writer = import(directory.path(), &fixture);
        check(&mut writer, &fixture);

        drop(writer);
        drop(directory);

        // The pipeline consumes the reorg boundary before publication. Compare
        // its writer here; a second writer invoked after publication no longer
        // sees that boundary and cannot follow a same-height fork replacement.
        for (branch, height) in [(4, 2), (2, 1), (1, 1)] {
            fixture.publish(branch, height);
            check_published(&fixture);
        }
    });
}
