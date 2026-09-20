use bitview_collections::Windows;
use bitview_transforms::SatsToCents;
use bitview_vecs::{
    LazyWindowStartVec, ValuePerBlockCumulative, ValuePerBlockCumulativeRolling, ValuePerBlockFull,
    WindowStarts,
};
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, StoredU64, Timestamp, TxIndex, Version};
use common::{indexes, init_cache, stored};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, BinaryTransform, Database, ImportableVec, PcoVec, ReadableVec,
    WritableVec,
};

mod common;

#[test]
fn full_value_retains_cumulative_rolling_versions_and_fiat_flows() {
    init_cache();
    let directory = tempdir().unwrap();
    let reference_directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let reference_db = Database::open(reference_directory.path()).unwrap();
    let indexes = indexes(&db);
    let timestamps = stored::<Height, _>(
        &db,
        "timestamps",
        (0..3).map(|i| Timestamp::from(i * 600_u32)),
    );
    let starts = LazyWindowStartVec::days("starts", Version::ONE, 1, &timestamps);
    let windows = WindowStarts(Windows {
        _24h: &starts,
        _1w: &starts,
        _1m: &starts,
        _1y: &starts,
    });
    let mut full =
        ValuePerBlockFull::forced_import(&db, "value", Version::new(11), &indexes, &windows)
            .unwrap();
    let reference = ValuePerBlockCumulativeRolling::forced_import(
        &reference_db,
        "value",
        Version::new(13),
        &indexes,
        &windows,
    )
    .unwrap();
    assert_eq!(
        full.cumulative.sats.height.name(),
        reference.cumulative.sats.height.name()
    );
    assert_eq!(
        full.cumulative.sats.height.version(),
        reference.cumulative.sats.height.version()
    );
    assert_eq!(full.block.sats.version(), reference.block.sats.version());
    let prices = stored::<Height, _>(
        &db,
        "prices",
        [100_000_000_u64, 200_000_000, 300_000_000].map(Cents::from),
    );
    let first = stored::<Height, _>(&db, "first_tx", (0..3usize).map(TxIndex::from));
    let counts = stored::<Height, _>(&db, "tx_counts", [StoredU64::from(1_u64); 3]);
    let mut amounts = PcoVec::<TxIndex, Sats>::forced_import(&db, "amounts", Version::ONE).unwrap();
    for sats in [1_u64, 2, 3].map(Sats::from) {
        amounts.push(sats);
    }
    amounts.write().unwrap();
    full.compute_from_indexes(
        Height::ZERO,
        &windows,
        &prices,
        &first,
        &counts,
        &amounts,
        &Exit::new(),
    )
    .unwrap();
    assert_eq!(
        full.cumulative.sats.height.collect_range_at(0, 3),
        [1_u64, 3, 6].map(Sats::from)
    );
    assert_eq!(
        full.cumulative.cents.height.collect_range_at(0, 3),
        [1_u64, 5, 14].map(Cents::from)
    );
    assert_eq!(
        full.block.cents.collect_range_at(0, 3),
        [1_u64, 4, 9].map(Cents::from)
    );
    assert_eq!(
        full.distribution
            .max
            ._24h
            .cents
            .height
            .collect_range_at(0, 3),
        [1_u64, 4, 9].map(Cents::from)
    );
}

#[test]
fn cumulative_values_reopen_resume_and_rewind_from_stored_totals() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = indexes(&db);
    let length = 16usize;
    let amounts: Vec<_> = (0..length).map(|i| Sats::from((i + 1) as u64)).collect();
    let prices: Vec<_> = (0..length)
        .map(|i| Cents::from((i as u64 + 1) * 100_000_000))
        .collect();
    let source = stored::<Height, _>(&db, "amounts", amounts.iter().copied());
    let price_source = stored::<Height, _>(&db, "prices", prices.iter().copied());
    let mut offset = 0usize;
    let first = stored::<Height, _>(
        &db,
        "first",
        (0..length).map(|i| {
            let start = TxIndex::from(offset);
            offset += i % 4;
            start
        }),
    );
    let counts = stored::<Height, _>(&db, "counts", (0..length).map(|i| StoredU64::from(i % 4)));
    let tx_amounts: Vec<_> = (0..offset)
        .map(|i| {
            if i % 5 == 0 {
                Sats::MAX
            } else {
                Sats::from(i as u64 + 1)
            }
        })
        .collect();
    let mut tx_source =
        PcoVec::<TxIndex, Sats>::forced_import(&db, "tx_amounts", Version::ONE).unwrap();
    for &sats in &tx_amounts {
        tx_source.push(sats);
    }
    tx_source.write().unwrap();

    for mode in 0..3 {
        let name = format!("cumulative_{mode}");
        let mut block_amounts = amounts.clone();
        if mode == 1 {
            block_amounts.iter_mut().for_each(|sats| *sats += *sats);
        } else if mode == 2 {
            let mut offset = 0;
            for (i, amount) in block_amounts.iter_mut().enumerate() {
                let end = offset + i % 4;
                *amount = tx_amounts[offset..end]
                    .iter()
                    .copied()
                    .filter(|sats| !sats.is_max())
                    .sum();
                offset = end;
            }
        }
        let block_cents: Vec<_> = block_amounts
            .iter()
            .copied()
            .zip(prices.iter().copied())
            .map(|(sats, cents)| SatsToCents::apply(sats, cents))
            .collect();
        let mut sum_sats = Sats::ZERO;
        let cumulative_sats: Vec<_> = block_amounts
            .iter()
            .map(|&sats| {
                sum_sats += sats;
                sum_sats
            })
            .collect();
        let mut sum_cents = Cents::ZERO;
        let cumulative_cents: Vec<_> = block_cents
            .iter()
            .map(|&cents| {
                sum_cents += cents;
                sum_cents
            })
            .collect();

        let mut output: ValuePerBlockCumulative =
            ValuePerBlockCumulative::forced_import(&db, &name, Version::ONE, &indexes).unwrap();
        for phase in 0..5 {
            if phase == 2 {
                drop(output);
                output = ValuePerBlockCumulative::forced_import(&db, &name, Version::ONE, &indexes)
                    .unwrap();
            }
            if phase == 4 {
                output
                    .cumulative
                    .sats
                    .height
                    .validate_computed_version_or_reset(Version::ZERO)
                    .unwrap();
            }
            let from = Height::from(if phase == 3 { 7usize } else { length });
            let exit = Exit::new();
            match mode {
                0 => output.compute_from(from, &price_source, &source, |_, sats| sats, &exit),
                1 => output.compute_from_pair(
                    from,
                    &price_source,
                    &source,
                    &source,
                    |_, a, b| a + b,
                    &exit,
                ),
                _ => output.compute_filtered_from_indexes(
                    from,
                    &price_source,
                    &first,
                    &counts,
                    &tx_source,
                    |sats| !sats.is_max(),
                    &exit,
                ),
            }
            .unwrap();
            assert_eq!(
                output.cumulative.sats.height.collect_range_at(0, length),
                cumulative_sats,
                "sats mode={mode} phase={phase}"
            );
            assert_eq!(
                output.cumulative.cents.height.collect_range_at(0, length),
                cumulative_cents,
                "cents mode={mode} phase={phase}"
            );
            assert_eq!(
                output.block.sats.collect_range_at(0, length),
                block_amounts,
                "block sats mode={mode} phase={phase}"
            );
            assert_eq!(
                output.block.cents.collect_range_at(0, length),
                block_cents,
                "block cents mode={mode} phase={phase}"
            );
        }
    }
}
