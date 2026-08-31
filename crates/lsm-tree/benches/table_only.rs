use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use lsm_tree::{CompressionType, Config, Tree, config::CompressionPolicy};

const ROWS: u32 = 100_000;
const RECOVERY_TABLES: u32 = 256;

fn populated_tree() -> (tempfile::TempDir, Tree) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let tree = Tree::open(Config::new(directory.path())).expect("tree");
    let mut ingestion = tree.ingestion().expect("ingestion");
    for key in 0..ROWS {
        ingestion
            .write(key.to_be_bytes(), (key * 2).to_be_bytes())
            .expect("write");
    }
    ingestion.finish().expect("finish ingestion");
    (directory, tree)
}

fn populated_recovery_tree() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary directory");
    let tree = Tree::open(Config::new(directory.path())).expect("tree");

    for key in 0..RECOVERY_TABLES {
        let mut ingestion = tree.ingestion().expect("ingestion");
        ingestion
            .write(key.to_be_bytes(), key.to_be_bytes())
            .expect("write");
        ingestion.finish().expect("finish ingestion");
    }

    assert_eq!(
        Some((RECOVERY_TABLES - 1).to_be_bytes().as_slice()),
        tree.get((RECOVERY_TABLES - 1).to_be_bytes())
            .expect("read")
            .as_deref(),
    );
    drop(tree);

    directory
}

fn table_only(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("table-only");
    group.throughput(Throughput::Elements(u64::from(ROWS)));

    group.bench_function("ingest", |bencher| {
        bencher.iter_batched(
            || {
                let directory = tempfile::tempdir().expect("temporary directory");
                let tree = Tree::open(Config::new(directory.path())).expect("tree");
                (directory, tree)
            },
            |(_directory, tree)| {
                let mut ingestion = tree.ingestion().expect("ingestion");
                for key in 0..ROWS {
                    ingestion
                        .write(key.to_be_bytes(), (key * 2).to_be_bytes())
                        .expect("write");
                }
                ingestion.finish().expect("finish ingestion");
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("compressed-ingest", |bencher| {
        bencher.iter_batched(
            || {
                let directory = tempfile::tempdir().expect("temporary directory");
                let config = Config::new(directory.path())
                    .data_block_compression_policy(CompressionPolicy::all(CompressionType::Lz4));
                let tree = Tree::open(config).expect("tree");
                (directory, tree)
            },
            |(_directory, tree)| {
                let mut ingestion = tree.ingestion().expect("ingestion");
                for key in 0..ROWS {
                    ingestion
                        .write(key.to_be_bytes(), (key * 2).to_be_bytes())
                        .expect("write");
                }
                ingestion.finish().expect("finish ingestion");
            },
            BatchSize::LargeInput,
        );
    });

    let (_directory, tree) = populated_tree();
    group.throughput(Throughput::Elements(u64::from(ROWS / 100)));
    group.bench_function("point-reads", |bencher| {
        bencher.iter(|| {
            for key in (0..ROWS).step_by(100) {
                std::hint::black_box(tree.get(key.to_be_bytes()).expect("read"));
            }
        });
    });
    group.throughput(Throughput::Elements(u64::from(ROWS)));
    group.bench_function("full-scan", |bencher| {
        bencher.iter(|| {
            let count = tree
                .iter()
                .map(|item| item.expect("read"))
                .map(std::hint::black_box)
                .count();
            assert_eq!(count, ROWS as usize);
        });
    });

    let recovery_directory = populated_recovery_tree();
    group.throughput(Throughput::Elements(u64::from(RECOVERY_TABLES)));
    group.bench_function("recovery-many-tables", |bencher| {
        bencher.iter(|| {
            std::hint::black_box(
                Tree::open(Config::new(recovery_directory.path())).expect("recover tree"),
            );
        });
    });
    group.finish();
}

criterion_group!(benches, table_only);
criterion_main!(benches);
