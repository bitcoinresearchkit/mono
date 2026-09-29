use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use bitcoin::Amount;
use bitview_plugin::{ComputePlugin, ImportContext, PluginData, UpdateContext};
use bitview_plugin_blocks::HasBlocks;
use bitview_plugin_distribution_age::{
    Dependencies as AgeDependencies, HasDistributionAge, Vecs as Age,
};
use bitview_plugin_distribution_size::{
    Dependencies as SizeDependencies, HasDistributionSize, Vecs as Size,
};
use bitview_plugin_indexer::HasIndexer;
use bitview_plugin_inputs::HasInputs;
use bitview_plugin_mappings::HasMappings;
use bitview_plugin_outputs::HasOutputs;
use bitview_plugin_price::HasPrice;
use bitview_plugin_utxo_history::HasUtxoHistory;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{CentsSats, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec,
    ReadableCloneableVec, WritableVec,
};

use super::chain_fixture::{ChainFixture, raw_fixture_block, run_genesis};

type Distribution = (Age, Size);

fn import(path: &Path, fixture: &ChainFixture) -> Distribution {
    let plugins = &fixture.plugins;
    let context = ImportContext::new(path);
    let windows = plugins.blocks().lookback.window_starts();
    let supply = plugins.utxo_history().supply.read_only_boxed_clone();
    let count = plugins.utxo_history().count.height.read_only_boxed_clone();
    (
        Age::import(
            context,
            plugins.mappings(),
            &windows,
            plugins.price(),
            &supply,
            &count,
        )
        .unwrap(),
        Size::import(
            context,
            plugins.mappings(),
            &windows,
            plugins.price(),
            &plugins.inputs().by_type,
            &plugins.outputs().by_type,
            &supply,
            &count,
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
    writer.1.compute(
        SizeDependencies {
            indexer: plugins.indexer(),
            mappings: plugins.mappings(),
            input_values: &plugins.inputs().value,
            price: plugins.price(),
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
}

fn check_published(fixture: &ChainFixture) {
    let directory = tempdir().unwrap();
    let mut fresh = import(directory.path(), fixture);
    compute(&mut fresh, fixture).unwrap();
    compare(fixture.plugins.distribution_age(), &fresh.0);
    compare(fixture.plugins.distribution_size(), &fresh.1);
}

fn checkpoint_path(writer: &Distribution) -> PathBuf {
    writer
        .1
        .addr_state
        .p2a
        .db_path()
        .join("changes")
        .join("cohort_caps/usize")
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
        let checkpoint = checkpoint_path(&writer);
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

        // Missing or incomplete scalar state cannot resume against newer address
        // state. Both cases must rebuild to the same result as a fresh writer.
        for truncate in [true, false] {
            let path = writer.1.addr_state.p2a.db_path();
            drop(writer);
            {
                let db = Database::open(&path).unwrap();
                let mut caps: MutableVec<BytesVec<usize, CentsSats>> =
                    MutableVec::forced_import_with(
                        ImportOptions::new(&db, "cohort_caps", Version::ONE)
                            .with_saved_stamped_changes(10),
                    )
                    .unwrap();
                if truncate {
                    caps.truncate_if_needed_at(40).unwrap();
                } else {
                    caps.validate_computed_version_or_reset(Version::ZERO)
                        .unwrap();
                }
                caps.flush().unwrap();
            }
            writer = import(directory.path(), &fixture);
            check(&mut writer, &fixture);
        }

        writer
            .0
            .coinblocks_destroyed
            .stored_mut()
            .any_truncate_if_needed_at(1)
            .unwrap();
        check(&mut writer, &fixture);
        writer
            .0
            .coindays_created
            .under_1h
            .cumulative
            .height
            .validate_computed_version_or_reset(Version::ZERO)
            .unwrap();
        check(&mut writer, &fixture);

        writer
            .0
            .age_bounds
            .stored
            .under_4m
            .min
            .truncate_if_needed_at(0)
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
        let checkpoint = checkpoint_path(&writer);
        let saved = directory.path().join("saved-checkpoints");

        // Fail after block processing has advanced the state and written vectors.
        // Failed vecdb writers must be discarded. Reopening must recover a
        // consistent scalar/address checkpoint or rebuild from genesis.
        fixture.publish(4, 2);
        fs::rename(&checkpoint, &saved).unwrap();
        fs::write(&checkpoint, b"blocked checkpoint directory").unwrap();
        assert!(compute(&mut writer, &fixture).is_err());
        fs::remove_file(&checkpoint).unwrap();
        fs::rename(&saved, &checkpoint).unwrap();
        drop(writer);
        let mut writer = import(directory.path(), &fixture);
        check(&mut writer, &fixture);
        drop(writer);
        check(&mut import(directory.path(), &fixture), &fixture);
    });
}
