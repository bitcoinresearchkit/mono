#[allow(dead_code)]
mod common;

#[cfg(test)]
mod tests {
    use super::common::init_cache;

    use bitview_transforms::RatioU64;
    use bitview_vecs::{LazyIndexedVec, LazyRollingRatioVec};
    use brk_types::{Height, PartsPerMillion32, StoredU64};
    use tempfile::tempdir;
    use vecdb::{
        AnyStoredVec, BinaryTransform, Database, EagerVec, ImportableVec, PcoVec, ReadableVec,
        ReverseOperands, Version, WritableVec,
    };
    #[cfg(feature = "diagnostics")]
    use vecdb::{Budgeted, ReadableCloneableVec};

    #[cfg(feature = "diagnostics")]
    use vecdb::diagnostics;

    #[cfg(feature = "diagnostics")]
    use super::common;

    #[test]
    fn derives_cumulative_and_rolling_ratios_from_stored_counts() {
        init_cache();
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut numerator: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "numerator", Version::ONE).unwrap();
        let mut denominator: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "denominator", Version::ONE).unwrap();
        let mut starts: EagerVec<PcoVec<Height, Height>> =
            EagerVec::forced_import(&db, "starts", Version::ONE).unwrap();

        for value in [10_u64, 30, 60, 100] {
            numerator.push(StoredU64::from(value));
        }
        for value in [20_u64, 50, 90, 140] {
            denominator.push(StoredU64::new(value));
        }
        for value in [0, 0, 1, 2] {
            starts.push(Height::new(value));
        }
        numerator.write().unwrap();
        denominator.write().unwrap();
        starts.write().unwrap();

        let cumulative = LazyIndexedVec::new(
            "cumulative",
            Version::ONE,
            &denominator,
            &numerator,
            |_, count, numerator| RatioU64::<PartsPerMillion32>::apply(numerator, count),
        );
        let rolling = LazyRollingRatioVec::<
            StoredU64,
            StoredU64,
            PartsPerMillion32,
            ReverseOperands<RatioU64<PartsPerMillion32>>,
        >::new("rolling", Version::ONE, &denominator, &numerator, &starts);

        assert_eq!(
            cumulative.collect_range_at(0, 4),
            [0.5, 0.6, 2.0 / 3.0, 5.0 / 7.0].map(PartsPerMillion32::from)
        );
        assert_eq!(
            cumulative.read_sorted_at(&[0, 2, 2, 4]),
            [0.5, 2.0 / 3.0, 2.0 / 3.0].map(PartsPerMillion32::from)
        );
        assert_eq!(
            rolling.collect_range_at(0, 4),
            [0.5, 0.6, 5.0 / 7.0, 7.0 / 9.0].map(PartsPerMillion32::from)
        );
        assert_eq!(
            rolling.read_sorted_at(&[0, 2, 2, 3, 4]),
            [0.5, 5.0 / 7.0, 5.0 / 7.0, 7.0 / 9.0].map(PartsPerMillion32::from)
        );
        assert_eq!(
            rolling.read_sorted_at(&[3]),
            [7.0 / 9.0].map(PartsPerMillion32::from)
        );
        #[cfg(feature = "diagnostics")]
        {
            const N: usize = 32_768;
            let mut numerator = EagerVec::<PcoVec<Height, StoredU64, Budgeted>>::forced_import(
                &db,
                "cold_numerator",
                Version::ONE,
            )
            .unwrap();
            for i in 0..N {
                numerator.push(StoredU64::from((i as u64 + 1) * 3));
            }
            numerator.write().unwrap();
            let counts = common::stored::<Height, _>(
                &db,
                "cold_counts",
                (1..=N).map(|i| StoredU64::from(i as u64)),
            );
            let starts = common::stored::<Height, _>(
                &db,
                "cold_starts",
                (0..N).map(|i| Height::from(i.saturating_sub(N / 2))),
            );
            counts.collect_one_at(N - 1).unwrap();
            let cumulative = LazyIndexedVec::new(
                "cold_cumulative",
                Version::ONE,
                &counts,
                &numerator,
                |_, count, numerator| RatioU64::<PartsPerMillion32>::apply(numerator, count),
            );
            let rolling =
                LazyRollingRatioVec::<
                    StoredU64,
                    StoredU64,
                    PartsPerMillion32,
                    ReverseOperands<RatioU64<PartsPerMillion32>>,
                >::new("cold_rolling", Version::ONE, &counts, &numerator, &starts);
            for view in [
                cumulative.read_only_boxed_clone(),
                rolling.read_only_boxed_clone(),
            ] {
                for sorted in [false, true] {
                    init_cache().clear();
                    // Eviction also clears metadata now; warm it before counting
                    // only the cold numerator's selective reads.
                    counts.collect();
                    starts.collect();
                    diagnostics::take();
                    let values = if sorted {
                        view.read_sorted_at(&[N - 8, N - 3, N - 1])
                    } else {
                        view.collect_range_at(N - 8, N)
                    };
                    assert_eq!(
                        values,
                        vec![PartsPerMillion32::from(3.0); if sorted { 3 } else { 8 }]
                    );
                    let decodes = diagnostics::take();
                    assert!(
                        decodes > 0 && decodes <= 4,
                        "short read decoded {decodes} chunks"
                    );
                }
            }
        }
    }
}
