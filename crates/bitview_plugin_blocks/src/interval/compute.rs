use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::{CheckedSub, Timestamp};
use vecdb::{Exit, ReadableVec};

use super::Vecs;

pub trait Compute {
    fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()>;
}

impl Compute for Vecs {
    fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
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
                    timestamp.checked_sub(previous).unwrap_or(Timestamp::ZERO)
                } else {
                    Timestamp::ZERO
                };
                prev_timestamp = Some(timestamp);
                interval
            },
            exit,
        )
    }
}
