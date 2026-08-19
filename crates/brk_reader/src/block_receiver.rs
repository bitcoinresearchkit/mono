use brk_error::Result;
use brk_types::ReadBlock;
use crossbeam::channel::Receiver;

/// Stream of parsed blocks produced by the block-reading pipeline.
#[derive(Clone)]
pub struct BlockReceiver(Receiver<Result<ReadBlock>>);

pub fn new(receiver: Receiver<Result<ReadBlock>>) -> BlockReceiver {
    BlockReceiver(receiver)
}

impl BlockReceiver {
    /// Blocks until each parsed block arrives, stopping when the pipeline closes.
    pub fn iter(&self) -> impl Iterator<Item = Result<ReadBlock>> + '_ {
        self.0.iter()
    }
}

impl Iterator for BlockReceiver {
    type Item = Result<ReadBlock>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.recv().ok()
    }
}
