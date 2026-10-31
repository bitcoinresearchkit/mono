use std::{sync::LazyLock, thread};

use brk_types::ReadBlock;
use crossbeam_channel::{Sender, bounded};

// Freeing an indexed block's transactions, which the reader's threads allocated, took ~7% of the
// indexing thread around block 820,000: a thread of its own frees them, a few blocks behind at
// most. If it cannot be created, blocks free in place.
static FREER: LazyLock<Option<Sender<ReadBlock>>> = LazyLock::new(|| {
    let (send, recv) = bounded::<ReadBlock>(4);
    thread::Builder::new()
        .name("indexer-free".into())
        .spawn(move || recv.into_iter().for_each(drop))
        .ok()
        .map(|_| send)
});

/// Frees `block` off the calling thread.
pub(super) fn free(block: ReadBlock) {
    if let Some(freer) = FREER.as_ref() {
        let _ = freer.send(block);
    }
}
