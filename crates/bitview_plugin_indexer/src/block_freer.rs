#[cfg(feature = "bench-cpu")]
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
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
        .spawn(move || {
            for block in recv {
                drop(block);
                #[cfg(feature = "bench-cpu")]
                PENDING.fetch_sub(1, Ordering::Relaxed);
            }
        })
        .ok()
        .map(|_| send)
});

/// Blocks handed to the freer and not yet freed.
#[cfg(feature = "bench-cpu")]
static PENDING: AtomicUsize = AtomicUsize::new(0);

/// Frees `block` off the calling thread.
pub(super) fn free(block: ReadBlock) {
    if let Some(freer) = FREER.as_ref() {
        #[cfg(feature = "bench-cpu")]
        PENDING.fetch_add(1, Ordering::Relaxed);
        let _ = freer.send(block);
    }
}

/// Returns once every block handed to the freer is freed.
#[cfg(feature = "bench-cpu")]
pub(super) fn drain() {
    while PENDING.load(Ordering::Relaxed) != 0 {
        thread::sleep(Duration::from_micros(100));
    }
}
