use bitview_cohort::{ByType, SpendableType, SpendableTypeId};
use bitview_collections::Windows;
use bitview_vecs::{CountTotal, OutputTypeCounts, SpendableTypeCounts, import_cached};
use brk_types::{Height, PartsPerMillion32, StoredU64, Version};
use common::init_cache;
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ReadOnlyClone, ReadableVec, WritableVec};

mod common;

#[test]
fn type_domains_share_the_engine_without_sharing_the_wrong_denominator() {
    init_cache();
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let indexes = common::indexes(&db);
    let starts = common::stored::<Height, _>(&db, "starts", [Height::ZERO; 3]);
    let windows = Windows {
        _24h: &starts,
        _1w: &starts,
        _1m: &starts,
        _1y: &starts,
    };
    let mut totals = common::stored::<Height, _>(&db, "totals", [1_u64, 4, 8].map(StoredU64::from));
    let cached = totals.read_only_clone();
    let version = Version::new(11);
    let selected = SpendableTypeId::ALL[0].output_type();

    let mut inputs = SpendableType::try_new(|id| {
        import_cached::<Height, StoredU64>(&db, &format!("inputs_{}", id.name()), version)
    })
    .unwrap();
    let mut outputs = ByType::try_new(|id| {
        import_cached::<Height, StoredU64>(&db, &format!("outputs_{}", id.name()), version)
    })
    .unwrap();
    for count in [0_u64, 1, 2] {
        let values =
            ByType::from_fn(|kind| StoredU64::new(if kind == selected { count } else { 0 }));
        for &id in SpendableTypeId::ALL {
            let kind = id.output_type();
            inputs.get_mut(kind).push(*values.spendable.get(kind));
        }
        for (kind, target) in outputs.iter_typed_mut() {
            target.push(*values.get(kind));
        }
    }
    for target in inputs.iter_mut().chain(outputs.iter_mut()) {
        target.write().unwrap();
    }
    let input = SpendableTypeCounts::from_cumulative_sources(
        CountTotal::from_transformed_source(
            "non_coinbase",
            version,
            &cached,
            |height, total| total - StoredU64::from(height.incremented()),
            &indexes,
            &windows,
        ),
        |name| format!("{name}_inputs"),
        version,
        &inputs,
        &indexes,
        &windows,
    );
    let mut output = OutputTypeCounts::from_cumulative_sources(
        CountTotal::from_source("all", version, &cached, &indexes, &windows),
        |name| format!("{name}_outputs"),
        version,
        &outputs,
        &indexes,
        &windows,
    );
    assert_eq!(input.by_type.iter().count(), 11);
    assert_eq!(output.by_type.iter_mut().count(), 12);
    let input_shares = input.lazy_shares(
        version,
        |name| format!("{name}_input_share"),
        &windows,
        &indexes,
    );
    let output_shares = output.lazy_shares(
        version,
        |name| format!("{name}_output_share"),
        &windows,
        &indexes,
    );
    assert_eq!(
        input.all.cumulative.height.collect(),
        [0_u64, 2, 5].map(StoredU64::from)
    );
    assert_eq!(
        output.all.cumulative.height.collect(),
        [1_u64, 4, 8].map(StoredU64::from)
    );
    assert_eq!(
        input_shares
            .get(selected)
            .cumulative
            .ppm
            .height
            .collect_one_at(2),
        Some(PartsPerMillion32::from(0.4))
    );
    assert_eq!(
        output_shares
            .get(selected)
            .cumulative
            .ppm
            .height
            .collect_one_at(2),
        Some(PartsPerMillion32::from(0.25))
    );

    // Only the source owner needs to invalidate rewritten totals.
    totals.truncate_if_needed_at(2).unwrap();
    totals.push(StoredU64::from(10_u64));
    totals.write().unwrap();
    assert_eq!(
        input_shares
            .get(selected)
            .cumulative
            .ppm
            .height
            .collect_one_at(2),
        Some(PartsPerMillion32::from(2.0 / 7.0))
    );
    assert_eq!(
        output_shares
            .get(selected)
            .cumulative
            .ppm
            .height
            .collect_one_at(2),
        Some(PartsPerMillion32::from(0.2))
    );

    // Replacing cumulative counts at the same length updates already-captured
    // block, cumulative, and ratio readers without a separate revision token.
    for source in [inputs.get_mut(selected), outputs.get_mut(selected)] {
        source.truncate_if_needed_at(2).unwrap();
        source.push(StoredU64::from(4u64));
        source.write().unwrap();
    }
    assert_eq!(
        input.by_type.get(selected).block.collect(),
        [0u64, 1, 3].map(StoredU64::from)
    );
    assert_eq!(
        output
            .by_type
            .get(selected)
            .block
            .read_sorted_at(&[0, 2, 2, 3]),
        [0u64, 3, 3].map(StoredU64::from)
    );
    assert_eq!(
        input_shares
            .get(selected)
            .cumulative
            .ppm
            .height
            .collect_one_at(2),
        Some(PartsPerMillion32::from(4.0 / 7.0))
    );
    assert_eq!(
        output_shares
            .get(selected)
            .cumulative
            .ppm
            .height
            .collect_one_at(2),
        Some(PartsPerMillion32::from(0.4))
    );
}
