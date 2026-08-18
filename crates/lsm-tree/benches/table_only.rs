use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use lsm_tree::{Config, Tree};

const ROWS: u32 = 100_000;

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
    group.finish();
}

criterion_group!(benches, table_only);
criterion_main!(benches);
