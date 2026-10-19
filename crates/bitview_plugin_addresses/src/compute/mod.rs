mod block_loop;
mod readers;
mod write;
pub use block_loop::process_chunk;
pub use readers::{AddrReaders, Workspace};
pub(crate) use write::UnpublishedState;

use bitview_compute::undo_chunks;
use bitview_distribution::replay;
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use brk_error::Result;

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
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
            exit,
            |vecs, addrs, heights, prices| {
                let output_heights = mappings.output_heights.load_full();
                let mut workspace = Workspace::new(indexer, &output_heights);
                // An ordinary update is one chunk, the protected tail, written with its undo.
                for (blocks, tail) in undo_chunks(heights) {
                    let mut from = blocks.start;
                    while from < blocks.end {
                        from = process_chunk(
                            vecs,
                            addrs,
                            indexer,
                            mappings,
                            &mut workspace,
                            from..blocks.end,
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
