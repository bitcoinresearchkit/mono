use byteview::ByteView;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};

const COMPARISONS: u64 = 100_000;

fn compare_repeated(left: &ByteView, right: &ByteView) {
    for _ in 0..COMPARISONS {
        std::hint::black_box(left.cmp(std::hint::black_box(right)));
    }
}

fn byteview(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("byteview-ord");
    group.throughput(Throughput::Elements(COMPARISONS));

    let inline_left = ByteView::from(*b"00000001");
    let inline_right = ByteView::from(*b"00000002");
    group.bench_function("inline", |bencher| {
        bencher.iter(|| compare_repeated(&inline_left, &inline_right));
    });

    let long_different_left = ByteView::from([b'a'; 64]);
    let long_different_right = ByteView::from([b'b'; 64]);
    group.bench_function("long-different-prefix", |bencher| {
        bencher.iter(|| compare_repeated(&long_different_left, &long_different_right));
    });

    let long_shared_left = ByteView::from([b'a'; 64]);
    let mut long_shared_right = [b'a'; 64];
    long_shared_right[63] = b'b';
    let long_shared_right = ByteView::from(long_shared_right);
    group.bench_function("long-shared-prefix", |bencher| {
        bencher.iter(|| compare_repeated(&long_shared_left, &long_shared_right));
    });

    group.finish();
}

criterion_group!(benches, byteview);
criterion_main!(benches);
