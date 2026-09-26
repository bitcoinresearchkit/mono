use std::collections::BTreeSet;

use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, IndexSources, LazyWindowStartVec, import_cached};
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::{
    Height, PartsPerMillion32, Sats, StoredBool, StoredU64, Timestamp, TxIndex, Version,
};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Budgeted, Database, LazyVec, PcoVecValue, ReadableCloneableVec, ReadableVec,
    VecIndex, WritableVec,
};

use super::{Vecs, forced_import};

fn replace<I: VecIndex, T: PcoVecValue>(
    target: &mut CachedSeries<I, T>,
    values: impl IntoIterator<Item = T>,
) {
    target.truncate_if_needed_at(0).unwrap();
    for value in values {
        target.push(value);
    }
    target.write().unwrap();
}

fn checkpoint(vecs: &mut Vecs) {
    vecs.count.stored_mut().write().unwrap();
    vecs.fees.stored_mut().write().unwrap();
    vecs.fee_share.ppm.height.flush().unwrap();
}

#[test]
fn inscription_fees_survive_partial_inputs_reorgs_reopen_and_version_changes() {
    let cache = Budgeted::init_global(16 * 1024 * 1024).unwrap();
    let directory = tempdir().unwrap();
    let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
    let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
    let context = ImportContext::new(directory.path());
    let indexer = Indexer::import(context, &reader).unwrap();
    let mappings = Mappings::import(context, &indexer).unwrap();
    let indexes: &IndexSources = &mappings;
    indexes
        .first_height
        .day1
        .mapping()
        .update_at(0, [0usize, 2, 3].map(Height::from));

    let sources = Database::open(&directory.path().join("sources")).unwrap();
    let mut first_tx = import_cached::<Height, TxIndex>(&sources, "first", Version::ONE).unwrap();
    let mut counts = import_cached::<Height, StoredU64>(&sources, "counts", Version::ONE).unwrap();
    let mut inscription_counts =
        import_cached::<Height, StoredU64>(&sources, "inscription_counts", Version::ONE).unwrap();
    let mut fees = import_cached::<TxIndex, Sats>(&sources, "fees", Version::ONE).unwrap();
    let mut flags = import_cached::<TxIndex, StoredBool>(&sources, "flags", Version::ONE).unwrap();
    let mut timestamps =
        import_cached::<Height, Timestamp>(&sources, "timestamps", Version::ONE).unwrap();
    replace(&mut first_tx, [0usize, 4, 7, 9].map(TxIndex::from));
    replace(&mut counts, [4u64, 3, 2, 1].map(StoredU64::from));
    replace(
        &mut timestamps,
        [0, 43_200, 86_400, 172_800].map(|seconds| Timestamp::new(1_600_000_000 + seconds)),
    );
    let windows = Windows {
        _24h: LazyWindowStartVec::days("day", Version::ONE, 1, &timestamps),
        _1w: LazyWindowStartVec::days("week", Version::ONE, 7, &timestamps),
        _1m: LazyWindowStartVec::days("month", Version::ONE, 30, &timestamps),
        _1y: LazyWindowStartVec::days("year", Version::ONE, 365, &timestamps),
    };
    let starts = Windows {
        _24h: &windows._24h,
        _1w: &windows._1w,
        _1m: &windows._1m,
        _1y: &windows._1y,
    };
    let path = directory.path().join("inscription_fees");
    let db = Database::open(&path).unwrap();
    let mut vecs = forced_import(&db, Version::ONE, indexes, &starts).unwrap();
    let exit = Exit::default();
    // Each block starts with a zero-fee coinbase. Block 0 has two marked
    // transactions; blocks 1 and 3 have none; block 2's fees are all marked.
    let all_fees = [0u64, 10, 20, 30, 0, 50, 50, 0, 90, 0];
    let all_flags = [
        false, true, false, true, false, false, false, false, true, false,
    ];
    let shares = [2.0 / 3.0, 0.0, 1.0, 0.0].map(PartsPerMillion32::from);
    for (from, fee_len, flag_len, count_len, expected) in [
        (0usize, 4, 4, 0, vec![]),
        (0, 4, 3, 4, vec![]),
        (0, 4, 4, 4, vec![40u64]),
        (1, 7, 7, 1, vec![40]),
        (1, 6, 7, 4, vec![40]),
        (1, 7, 6, 4, vec![40]),
        (1, 7, 7, 4, vec![40, 40]),
        (2, 9, 8, 4, vec![40, 40]),
        (2, 8, 9, 4, vec![40, 40]),
        (2, 9, 9, 4, vec![40, 40, 130]),
        (3, 10, 10, 4, vec![40, 40, 130, 130]),
        (4, 10, 10, 4, vec![40, 40, 130, 130]),
    ] {
        replace(
            &mut inscription_counts,
            [2u64, 0, 1, 0][..count_len]
                .iter()
                .copied()
                .map(StoredU64::from),
        );
        replace(
            &mut fees,
            all_fees[..fee_len].iter().copied().map(Sats::from),
        );
        replace(
            &mut flags,
            all_flags[..flag_len].iter().copied().map(StoredBool::from),
        );
        vecs.compute_fees(
            Height::from(from),
            &first_tx,
            &counts,
            &inscription_counts,
            &flags,
            &fees,
            &exit,
        )
        .unwrap();
        checkpoint(&mut vecs);
        cache.clear();
        assert_eq!(
            vecs.fees.cumulative.height.collect(),
            expected.iter().copied().map(Sats::from).collect::<Vec<_>>()
        );
        assert_eq!(
            vecs.fee_share.ppm.height.collect(),
            shares[..expected.len()]
        );
    }
    assert_eq!(vecs.fees.block.collect(), [40u64, 0, 90, 0].map(Sats::from));
    assert_eq!(
        vecs.fees.sum._24h.height.collect(),
        [40u64, 40, 90, 0].map(Sats::from)
    );
    for window in [&vecs.fees.sum._1w, &vecs.fees.sum._1m, &vecs.fees.sum._1y] {
        assert_eq!(
            window.height.collect(),
            [40u64, 40, 130, 130].map(Sats::from)
        );
    }
    assert_eq!(
        vecs.fees.cumulative.day1.collect(),
        [40u64, 130, 130].map(|sats| Some(Sats::from(sats)))
    );
    assert_eq!(
        vecs.fees.sum._24h.resolutions.day1.collect(),
        [40u64, 90, 0].map(|sats| Some(Sats::from(sats)))
    );
    assert_eq!(
        vecs.fee_share.percent.day1.collect(),
        [0.0f32, 100.0, 0.0].map(|value| Some(value.into()))
    );

    let names: BTreeSet<_> = vecs
        .iter_any_visible()
        .map(|series| series.name().to_owned())
        .collect();
    assert_eq!(
        names,
        [
            "tx_count_inscription",
            "tx_count_inscription_cumulative",
            "tx_count_inscription_sum_24h",
            "tx_count_inscription_sum_1w",
            "tx_count_inscription_sum_1m",
            "tx_count_inscription_sum_1y",
            "tx_count_inscription_average_24h",
            "tx_count_inscription_average_1w",
            "tx_count_inscription_average_1m",
            "tx_count_inscription_average_1y",
            "inscription_fees",
            "inscription_fees_cumulative",
            "inscription_fees_sum_24h",
            "inscription_fees_sum_1w",
            "inscription_fees_sum_1m",
            "inscription_fees_sum_1y",
            "inscription_fees_average_24h",
            "inscription_fees_average_1w",
            "inscription_fees_average_1m",
            "inscription_fees_average_1y",
            "inscription_fee_share",
            "inscription_fee_share_ratio",
            "inscription_fee_share_ppm",
        ]
        .map(str::to_owned)
        .into_iter()
        .collect()
    );

    // Reopening preserves both the stored totals and their public lazy views.
    drop((vecs, db));
    let db = Database::open(&path).unwrap();
    let mut vecs = forced_import(&db, Version::ONE, indexes, &starts).unwrap();
    assert_eq!(vecs.fees.block.collect(), [40u64, 0, 90, 0].map(Sats::from));

    // Reorg changes both membership and fees, preserving only block 0.
    replace(
        &mut inscription_counts,
        [2u64, 1, 0, 0].map(StoredU64::from),
    );
    replace(
        &mut fees,
        [0u64, 10, 20, 30, 0, 100, 100, 0, 90, 0].map(Sats::from),
    );
    replace(
        &mut flags,
        [
            false, true, false, true, false, true, false, false, false, false,
        ]
        .map(StoredBool::from),
    );
    vecs.compute_fees(
        Height::from(1usize),
        &first_tx,
        &counts,
        &inscription_counts,
        &flags,
        &fees,
        &exit,
    )
    .unwrap();
    checkpoint(&mut vecs);
    assert_eq!(
        vecs.fees.cumulative.height.collect(),
        [40u64, 140, 140, 140].map(Sats::from)
    );
    assert_eq!(
        vecs.fee_share.ppm.height.collect(),
        [2.0 / 3.0, 0.5, 0.0, 0.0].map(PartsPerMillion32::from)
    );

    // A shorter chain truncates both outputs even when no new block is computed.
    replace(&mut first_tx, [0usize, 4].map(TxIndex::from));
    replace(&mut counts, [4u64, 3].map(StoredU64::from));
    vecs.compute_fees(
        Height::from(2usize),
        &first_tx,
        &counts,
        &inscription_counts,
        &flags,
        &fees,
        &exit,
    )
    .unwrap();
    checkpoint(&mut vecs);
    assert_eq!(
        vecs.fees.cumulative.height.collect(),
        [40u64, 140].map(Sats::from)
    );
    assert_eq!(
        vecs.fee_share.ppm.height.collect(),
        [2.0 / 3.0, 0.5].map(PartsPerMillion32::from)
    );

    // A changed detector version rebuilds history even when resuming at the tip.
    let revised_flags = LazyVec::init(
        "revised_flags",
        Version::TWO,
        flags.read_only_boxed_clone(),
        |_, flag: StoredBool| StoredBool::from(!flag.is_true()),
    );
    replace(&mut inscription_counts, [2u64, 2].map(StoredU64::from));
    vecs.compute_fees(
        Height::from(2usize),
        &first_tx,
        &counts,
        &inscription_counts,
        &revised_flags,
        &fees,
        &exit,
    )
    .unwrap();
    checkpoint(&mut vecs);
    assert_eq!(
        vecs.fees.cumulative.height.collect(),
        [20u64, 120].map(Sats::from)
    );
    assert_eq!(
        vecs.fee_share.ppm.height.collect(),
        [1.0 / 3.0, 0.5].map(PartsPerMillion32::from)
    );
}
