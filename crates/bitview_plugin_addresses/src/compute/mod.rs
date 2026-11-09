mod block_loop;
mod readers;
mod write;
pub use block_loop::process_chunk;
pub use readers::{AddrReaders, Workspace};
pub(crate) use write::UnpublishedState;

use bitview_compute::{WRITE_INTERVAL, undo_tail_start};
use bitview_distribution::replay;
use bitview_plugin::{ComputePlugin, DependentContext, compute_items};
use brk_error::Result;

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(&mut self, deps: Dependencies<'_>, context: DependentContext<'_>) -> Result<()> {
        let start = context.start().height;
        self.discard_unpublished_state();
        let version = deps.version();
        let Dependencies {
            indexer,
            mappings,
            price,
        } = deps;
        let exit = context.exit();
        replay::run(
            self,
            version,
            indexer,
            &price.spot.cents.height,
            start,
            exit,
            |vecs, addrs, heights, prices| {
                let output_heights = mappings.output_heights.load_full();
                let mut workspace = Workspace::new(indexer, &output_heights);
                // The bulk, then the protected tail, written with its undo: an ordinary update is
                // the tail alone. A flush rewrites most of the address state's pages however few
                // blocks it follows, so the bulk flushes once the cache fills or `WRITE_INTERVAL`
                // blocks after its last flush, not also at fixed boundaries.
                let tail = undo_tail_start(&heights);
                let chunks = [
                    (heights.start..tail, false),
                    (tail..heights.end, heights.end > 1),
                ];
                for (blocks, tail) in chunks {
                    let mut from = blocks.start;
                    while from < blocks.end {
                        from = process_chunk(
                            vecs,
                            addrs,
                            indexer,
                            mappings,
                            &mut workspace,
                            from..blocks.end.min(from + WRITE_INTERVAL),
                            prices,
                            tail,
                            exit,
                        )?;
                    }
                }
                Ok(())
            },
        )
    }
}
