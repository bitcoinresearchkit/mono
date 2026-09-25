use std::sync::LazyLock;

use rayon::{ThreadPool, ThreadPoolBuilder, current_num_threads, join as rayon_join};

use crate::processor::transaction::ComputedTx;

// Blocking lookups can use extra workers while others wait on storage.
// If those workers cannot be created, the existing pool remains usable.
static READ_POOL: LazyLock<Option<ThreadPool>> = LazyLock::new(|| {
    ThreadPoolBuilder::new()
        .num_threads(current_num_threads().saturating_mul(2))
        .thread_name(|index| format!("indexer-read-{index}"))
        .build()
        .ok()
});

pub(super) fn join<A, B, RA, RB>(txs: &[ComputedTx<'_>], a: A, b: B) -> (RA, RB)
where
    A: FnOnce() -> RA + Send,
    B: FnOnce() -> RB + Send,
    RA: Send,
    RB: Send,
{
    let records = txs.last().map_or(0, |tx| {
        tx.input_offset + tx.tx.input.len() + tx.output_offset + tx.tx.output.len()
    });

    if records > current_num_threads()
        && let Some(pool) = READ_POOL.as_ref()
    {
        pool.install(|| rayon_join(a, b))
    } else {
        rayon_join(a, b)
    }
}
