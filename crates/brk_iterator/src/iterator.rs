use brk_error::{Error, Result};
use brk_types::Block;

use crate::State;

pub struct BlockIterator(State);

impl BlockIterator {
    pub fn new(state: State) -> Self {
        Self(state)
    }
}

impl Iterator for BlockIterator {
    type Item = Result<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            State::Rpc {
                client,
                heights,
                prev_hash,
            } => {
                let height = heights.next()?;
                let hash = match client.get_block_hash(height) {
                    Ok(h) => h,
                    Err(e) => return Some(Err(e)),
                };
                let block = match client.get_block(&hash) {
                    Ok(b) => b,
                    Err(e) => return Some(Err(e)),
                };

                if prev_hash
                    .as_ref()
                    .is_some_and(|prev| block.header.prev_blockhash != prev.into())
                {
                    return Some(Err(Error::Internal(
                        "rpc iterator: chain continuity broken (likely reorg mid-iteration)",
                    )));
                }

                prev_hash.replace(hash);

                Some(Ok(Block::from((height, hash, block))))
            }
            State::Reader { receiver } => match receiver.next()? {
                Ok(b) => Some(Ok(Block::from(b))),
                Err(e) => Some(Err(e)),
            },
        }
    }
}
