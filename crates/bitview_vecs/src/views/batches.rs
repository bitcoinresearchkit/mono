use vecdb::READ_CHUNK_SIZE;

/// Values per bounded input batch, as in vecdb's lazy delta views.
const BATCH_SIZE: usize = 16 * READ_CHUNK_SIZE;

/// Splits `[from, to)` into batches of about `BATCH_SIZE` values made of whole source chunks:
/// after the first, every batch starts on a source page boundary, so no page is decoded twice.
/// Input scratch then scales with a batch instead of the whole request.
pub(super) fn batches(
    from: usize,
    to: usize,
    chunk_size: usize,
) -> impl Iterator<Item = (usize, usize)> {
    let chunk_size = chunk_size.max(1);
    let size = chunk_size * (BATCH_SIZE / chunk_size).max(1);
    let mut at = from;
    std::iter::from_fn(move || {
        (at < to).then(|| {
            let end = (at + size - at % size).min(to);
            let batch = (at, end);
            at = end;
            batch
        })
    })
}
