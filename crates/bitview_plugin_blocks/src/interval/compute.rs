use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use bitview_primitives::Seconds;
use brk_exit::Exit;
use vecdb::ReadableVec;

use super::Vecs;

impl Vecs {
    pub fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let mut prev_timestamp = None;
        self.0.compute_from(
            starting_height,
            &indexer.vecs().blocks.timestamp,
            |height, timestamp| {
                let interval = if let Some(previous_height) = height.decremented() {
                    let previous = prev_timestamp.unwrap_or_else(|| {
                        indexer
                            .vecs()
                            .blocks
                            .timestamp
                            .collect_one(previous_height)
                            .unwrap()
                    });
                    Seconds::new((*timestamp).saturating_sub(*previous))
                } else {
                    Seconds::ZERO
                };
                prev_timestamp = Some(timestamp);
                interval
            },
            exit,
        )
    }
}
