use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use lsm_tree::{CompressionType, Config, Tree, config::CompressionPolicy};

const ROWS: u32 = 100_000;
const GENERATIONS: u32 = 4;
const LOOKUPS: u64 = 1_000;
const MISSING_KEY: u32 = 50_008;
const TOMBSTONED_KEY: u32 = 50_004;
const NEWEST_KEY: u32 = 50_011;
const OLDEST_KEY: u32 = 50_012;
const OUTSIDE_KEY: u32 = ROWS + 1;
const MAINTENANCE_ROWS: u32 = 10_000;
const MAINTENANCE_GENERATIONS: u32 = 8;

fn populated_multirun_tree() -> (tempfile::TempDir, Tree) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let tree = Tree::open(Config::new(directory.path())).expect("tree");

    for generation in 0..GENERATIONS {
        let mut ingestion = tree.ingestion().expect("ingestion");

        for key in 0..ROWS {
            if generation == GENERATIONS - 1 && key == TOMBSTONED_KEY {
                ingestion
                    .write_weak_tombstone(key.to_be_bytes())
                    .expect("write tombstone");
            } else if key != MISSING_KEY && key % GENERATIONS == generation {
                ingestion
                    .write(key.to_be_bytes(), generation.to_be_bytes())
                    .expect("write");
            }
        }

        ingestion.finish().expect("finish ingestion");
    }

    assert_eq!(GENERATIONS as usize, tree.l0_run_count());
    assert_eq!(
        Some((GENERATIONS - 1).to_be_bytes().as_slice()),
        tree.get(NEWEST_KEY.to_be_bytes())
            .expect("newest read")
            .as_deref(),
    );
    assert_eq!(
        Some(0_u32.to_be_bytes().as_slice()),
        tree.get(OLDEST_KEY.to_be_bytes())
            .expect("oldest read")
            .as_deref(),
    );
    assert_eq!(
        None,
        tree.get(MISSING_KEY.to_be_bytes()).expect("missing read")
    );
    assert_eq!(
        None,
        tree.get(TOMBSTONED_KEY.to_be_bytes())
            .expect("tombstone read")
    );

    (directory, tree)
}

fn point_reads(tree: &Tree, key: u32) {
    for _ in 0..LOOKUPS {
        std::hint::black_box(
            tree.get(std::hint::black_box(key.to_be_bytes()))
                .expect("read"),
        );
    }
}

fn multirun(criterion: &mut Criterion) {
    let (_directory, tree) = populated_multirun_tree();
    let mut group = criterion.benchmark_group("multirun-uncompacted");
    group.throughput(Throughput::Elements(LOOKUPS));

    group.bench_function("newest-hit", |bencher| {
        bencher.iter(|| point_reads(&tree, NEWEST_KEY));
    });
    group.bench_function("oldest-hit", |bencher| {
        bencher.iter(|| point_reads(&tree, OLDEST_KEY));
    });
    group.bench_function("deep-miss", |bencher| {
        bencher.iter(|| point_reads(&tree, MISSING_KEY));
    });
    group.bench_function("weak-tombstone", |bencher| {
        bencher.iter(|| point_reads(&tree, TOMBSTONED_KEY));
    });
    group.bench_function("outside-range-miss", |bencher| {
        bencher.iter(|| point_reads(&tree, OUTSIDE_KEY));
    });

    group.throughput(Throughput::Elements(u64::from(ROWS)));
    group.bench_function("full-scan", |bencher| {
        bencher.iter(|| {
            let count = tree
                .iter()
                .map(|item| item.expect("read"))
                .map(std::hint::black_box)
                .count();
            assert_eq!(ROWS as usize - 2, count);
        });
    });
    group.finish();
}

fn maintenance_workload(reads_per_generation: u32, compressed: bool) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let config = Config::new(directory.path());
    let config = if compressed {
        config.data_block_compression_policy(CompressionPolicy::all(CompressionType::Lz4))
    } else {
        config
    };
    let tree = Tree::open(config).expect("tree");
    let missing_key = MAINTENANCE_ROWS / 2;

    for generation in 0..MAINTENANCE_GENERATIONS {
        let mut ingestion = tree.ingestion().expect("ingestion");
        for key in 0..MAINTENANCE_ROWS {
            if key != missing_key {
                ingestion
                    .write(key.to_be_bytes(), generation.to_be_bytes())
                    .expect("write");
            }
        }
        ingestion.finish().expect("finish ingestion");

        for read in 0..reads_per_generation {
            let key = if read % 2 == 0 {
                missing_key
            } else {
                read % MAINTENANCE_ROWS
            };
            std::hint::black_box(tree.get(key.to_be_bytes()).expect("read"));
        }

        tree.compact().expect("compact");
    }

    assert_eq!(
        Some((MAINTENANCE_GENERATIONS - 1).to_be_bytes().as_slice()),
        tree.get(0_u32.to_be_bytes())
            .expect("final read")
            .as_deref(),
    );
}

fn maintenance(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("multirun-maintenance");
    group.bench_function("write-only", |bencher| {
        bencher.iter(|| maintenance_workload(0, false));
    });
    group.bench_function("compressed-write-only", |bencher| {
        bencher.iter(|| maintenance_workload(0, true));
    });
    group.bench_function("read-heavy", |bencher| {
        bencher.iter(|| maintenance_workload(10_000, false));
    });
    group.bench_function("read-dominant", |bencher| {
        bencher.iter(|| maintenance_workload(100_000, false));
    });
    group.finish();
}

criterion_group!(benches, multirun, maintenance);
criterion_main!(benches);
