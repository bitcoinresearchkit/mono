use std::{collections::BTreeMap, fs, path::Path};

use bitcoin::Amount;
use bitview_plugin::{ComputePlugin, ImportContext, PluginData, UpdateContext};
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_distribution::{Dependencies, HasDistribution, Vecs as Distribution};
use bitview_plugin_indexer::HasIndexer;
use bitview_plugin_inputs::HasInputs;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_outputs::HasOutputs;
use bitview_plugin_price::HasPrice;
use bitview_plugin_transactions::HasTransactions;
use bitview_urpd::AgeRangeUrpds;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Version;
use tempfile::tempdir;
use vecdb::{ReadableVec, WritableVec};

use super::chain_fixture::{ChainFixture, raw_fixture_block, run_genesis};

fn import(path: &Path, fixture: &ChainFixture) -> Distribution {
    Distribution::import(
        ImportContext::new(path),
        fixture.plugins.mappings(),
        &fixture.plugins.blocks().lookback.window_starts(),
        fixture.plugins.price(),
        &fixture.plugins.inputs().by_type,
        &fixture.plugins.outputs().by_type,
    )
    .unwrap()
}

fn compute(writer: &mut Distribution, fixture: &ChainFixture) -> Result<AgeRangeUrpds> {
    writer.compute(
        Dependencies {
            indexer: fixture.plugins.indexer(),
            mappings: fixture.plugins.mappings(),
            inputs: fixture.plugins.inputs(),
            outputs: fixture.plugins.outputs(),
            transactions: fixture.plugins.transactions(),
            price: fixture.plugins.price(),
        },
        UpdateContext::new(&Exit::default()),
    )
}

fn series(writer: &Distribution) -> BTreeMap<String, Vec<u8>> {
    let mut result = BTreeMap::new();
    writer.for_each_visible(&mut |vec| {
        let mut values = Vec::new();
        vec.write_json(None, None, &mut values).unwrap();
        result.insert(vec.region_name(), values);
    });
    result
}

fn snapshots(writer: &Distribution) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(AgeRangeUrpds::dir(&writer.states_path))
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().into_string().unwrap(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect()
}

fn check(writer: &mut Distribution, fixture: &ChainFixture) {
    let actual = compute(writer, fixture).unwrap();
    let directory = tempdir().unwrap();
    let mut fresh = import(directory.path(), fixture);
    let expected = compute(&mut fresh, fixture).unwrap();
    assert_eq!(
        actual.iter().collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
    compare(writer, &fresh);
}

fn check_published(fixture: &ChainFixture) {
    let directory = tempdir().unwrap();
    let mut fresh = import(directory.path(), fixture);
    compute(&mut fresh, fixture).unwrap();
    compare(fixture.plugins.distribution(), &fresh);
}

fn compare(writer: &Distribution, fresh: &Distribution) {
    assert_eq!(
        writer
            .supply_state
            .collect()
            .iter()
            .map(|s| (s.utxo_count, s.value))
            .collect::<Vec<_>>(),
        fresh
            .supply_state
            .collect()
            .iter()
            .map(|s| (s.utxo_count, s.value))
            .collect::<Vec<_>>()
    );
    assert_eq!(snapshots(writer), snapshots(fresh));
    let expected = series(fresh);
    for (name, values) in series(writer) {
        assert_eq!(values, expected[&name], "{name}");
    }
}

#[test]
fn distribution_live_state_matches_rebuild_after_append_reopen_reorg_and_failure() {
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

        // A clean update needs no cohort checkpoint reads. Temporarily hide one
        // checkpoint directory; rebuilding would recreate it.
        let checkpoint = writer.states_path.join("utxos_under_1h_old/cost_basis");
        let saved = directory.path().join("saved-checkpoints");
        fs::rename(&checkpoint, &saved).unwrap();
        check(&mut writer, &fixture);
        assert!(!checkpoint.exists());
        fs::rename(&saved, &checkpoint).unwrap();

        for (branch, height) in [(1, 1), (4, 2), (5, 3), (6, 4)] {
            fixture.publish(branch, height);
            check(&mut writer, &fixture);
        }
        drop(writer);
        let mut writer = import(directory.path(), &fixture);
        check(&mut writer, &fixture);

        writer
            .coinblocks_destroyed
            .stored_mut()
            .any_truncate_if_needed_at(1)
            .unwrap();
        check(&mut writer, &fixture);
        writer
            .supply_state
            .validate_computed_version_or_reset(Version::ZERO)
            .unwrap();
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

        let directory = tempdir().unwrap();
        let mut writer = import(directory.path(), &fixture);
        check(&mut writer, &fixture);
        let checkpoint = writer.states_path.join("utxos_under_1h_old/cost_basis");
        let saved = directory.path().join("saved-checkpoints");

        // Fail after block processing has advanced the state and written vectors.
        // Retry must recover from disk, discarding the partially advanced state.
        fixture.publish(4, 2);
        fs::rename(&checkpoint, &saved).unwrap();
        fs::write(&checkpoint, b"blocked checkpoint directory").unwrap();
        assert!(compute(&mut writer, &fixture).is_err());
        fs::remove_file(&checkpoint).unwrap();
        fs::rename(&saved, &checkpoint).unwrap();
        check(&mut writer, &fixture);
        drop(writer);
        check(&mut import(directory.path(), &fixture), &fixture);
    });
}
