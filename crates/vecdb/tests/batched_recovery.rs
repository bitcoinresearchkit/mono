use brk_exit::Exit;
use tempfile::tempdir;
#[cfg(feature = "lz4")]
use vecdb::LZ4Vec;
#[cfg(feature = "pco")]
use vecdb::PcoVec;
#[cfg(feature = "zerocopy")]
use vecdb::ZeroCopyVec;
#[cfg(feature = "zstd")]
use vecdb::ZstdVec;
use vecdb::{
    AnyStoredVec, AnyVec, Budgeted, BytesVec, Database, EagerVec, ImportableVec, ReadableVec,
    StoredVec, Version, WritableVec,
};

#[derive(Clone, Copy, Debug)]
enum Compute {
    Batched,
    Transform,
}

fn check_recovery<V: StoredVec<I = usize, T = u64>>(compute: Compute) {
    init_cache();
    let exit = Exit::new();
    for max_from in [3, 99] {
        let directory = tempdir().unwrap();
        let mut previous_values: &[u64] = &[];
        for (expected, version) in [
            (&[0, 1, 2, 3, 4][..], Version::ONE),
            (&[0, 1, 2][..], Version::ONE),
            (&[0, 1, 2, 103, 104][..], Version::ONE),
            (&[][..], Version::ONE),
            (&[][..], Version::TWO),
            (&[200, 201, 202, 203, 204][..], Version::TWO),
        ] {
            let to = expected.len();
            let dependency_version;
            {
                let db = Database::open(directory.path()).unwrap();
                let mut source =
                    EagerVec::<BytesVec<usize, u64>>::forced_import(&db, "source", version)
                        .unwrap();
                source.truncate_if_needed_at(0).unwrap();
                for &value in expected {
                    source.push(value);
                }
                source.write().unwrap();
                dependency_version = source.version();
                let mut output = EagerVec::<V>::forced_import(&db, "output", Version::ONE).unwrap();
                let previous = output.collect();
                let start =
                    if output.version() == output.header().vec_version() + dependency_version {
                        output.len().min(max_from).min(to)
                    } else {
                        0
                    };
                let mut appended = 0;
                match compute {
                    Compute::Batched => output.compute_batched_to(
                        max_from,
                        to,
                        dependency_version,
                        2,
                        |output, range| {
                            for index in range {
                                output.push(expected[index]);
                                appended += 1;
                            }
                            Ok(())
                        },
                        &exit,
                    ),
                    Compute::Transform => output.compute_transform(
                        max_from,
                        &source,
                        |(index, value, _)| {
                            appended += 1;
                            (index, value)
                        },
                        &exit,
                    ),
                }
                .unwrap();
                assert_eq!(appended, to - start, "{compute:?}: appended rows");
                assert_eq!(output.len(), to);
                assert_eq!(output.collect().as_slice(), expected);
                assert_eq!(previous.as_slice(), previous_values);
                // No extra write/flush: the compute call must publish truncation itself.
            }
            let db = Database::open(directory.path()).unwrap();
            let output = EagerVec::<V>::forced_import(&db, "output", Version::ONE).unwrap();
            assert_eq!(output.len(), to, "reopened length at max_from={max_from}");
            assert_eq!(output.collect_range_at(0, to), expected);
            assert_eq!(
                output.version(),
                output.header().vec_version() + dependency_version
            );
            previous_values = expected;
        }
    }
}

#[test]
fn raw_batched_recovery() {
    check_recovery::<BytesVec<usize, u64>>(Compute::Batched);
    check_recovery::<BytesVec<usize, u64, Budgeted>>(Compute::Batched);
}

#[cfg(feature = "pco")]
#[test]
fn pco_batched_recovery() {
    check_recovery::<PcoVec<usize, u64>>(Compute::Batched);
    check_recovery::<PcoVec<usize, u64, Budgeted>>(Compute::Batched);
}

#[cfg(feature = "lz4")]
#[test]
fn lz4_batched_recovery() {
    check_recovery::<LZ4Vec<usize, u64>>(Compute::Batched);
    check_recovery::<LZ4Vec<usize, u64, Budgeted>>(Compute::Batched);
}

#[cfg(feature = "zstd")]
#[test]
fn zstd_batched_recovery() {
    check_recovery::<ZstdVec<usize, u64>>(Compute::Batched);
    check_recovery::<ZstdVec<usize, u64, Budgeted>>(Compute::Batched);
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy_batched_recovery() {
    check_recovery::<ZeroCopyVec<usize, u64>>(Compute::Batched);
    check_recovery::<ZeroCopyVec<usize, u64, Budgeted>>(Compute::Batched);
}

#[test]
fn zero_append_compute_transform_recovery() {
    check_recovery::<BytesVec<usize, u64>>(Compute::Transform);
    #[cfg(feature = "pco")]
    check_recovery::<PcoVec<usize, u64>>(Compute::Transform);
    #[cfg(feature = "lz4")]
    check_recovery::<LZ4Vec<usize, u64>>(Compute::Transform);
    #[cfg(feature = "zstd")]
    check_recovery::<ZstdVec<usize, u64>>(Compute::Transform);
    #[cfg(feature = "zerocopy")]
    check_recovery::<ZeroCopyVec<usize, u64>>(Compute::Transform);
}

#[allow(dead_code)]
#[path = "common/cache.rs"]
mod cache;
use cache::init_cache;
