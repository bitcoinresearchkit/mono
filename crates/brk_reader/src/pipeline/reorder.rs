use std::cmp::Ordering;

use brk_error::{Error, Result};
use brk_types::ReadBlock;
use crossbeam::channel::Sender;
use rustc_hash::FxHashMap;

use super::forward::MAX_OUTSTANDING_BYTES;

pub struct ReorderState {
    pub next_offset: u32,
    /// Parsed blocks past a gap, with their raw lengths.
    pending: FxHashMap<u32, (ReadBlock, usize)>,
    /// Raw bytes of the blocks dispatched and not emitted yet: at most
    /// [`MAX_OUTSTANDING_BYTES`] past the block emission waits for.
    outstanding: usize,
    send_to_consumer: Sender<Result<ReadBlock>>,
    consumer_dropped: bool,
}

impl ReorderState {
    pub fn new(send_to_consumer: Sender<Result<ReadBlock>>) -> Self {
        Self {
            next_offset: 0,
            pending: FxHashMap::default(),
            outstanding: 0,
            send_to_consumer,
            consumer_dropped: false,
        }
    }

    pub fn finalize(self, expected_count: usize) -> Result<()> {
        if !self.consumer_dropped && (self.next_offset as usize) < expected_count {
            return Err(Error::Internal(
                "forward pipeline: blk files missing canonical blocks",
            ));
        }
        Ok(())
    }

    /// Takes credit for dispatching `len` raw bytes at `offset`: within the
    /// budget, or the block emission waits for.
    pub fn admit(&mut self, offset: u32, len: usize) -> bool {
        if offset != self.next_offset && self.outstanding + len > MAX_OUTSTANDING_BYTES {
            return false;
        }
        self.outstanding += len;
        true
    }

    /// `len`: the block's raw length, credited back once it is emitted.
    pub fn try_emit(&mut self, offset: u32, block: ReadBlock, len: usize) -> bool {
        match offset.cmp(&self.next_offset) {
            Ordering::Equal => {
                if !self.send(block, len) {
                    return false;
                }
                while let Some((next, len)) = self.pending.remove(&self.next_offset) {
                    if !self.send(next, len) {
                        return false;
                    }
                }
                true
            }
            Ordering::Greater => {
                self.pending.insert(offset, (block, len));
                true
            }
            Ordering::Less => true,
        }
    }

    fn send(&mut self, block: ReadBlock, len: usize) -> bool {
        self.outstanding -= len;
        if self.send_to_consumer.send(Ok(block)).is_err() {
            self.consumer_dropped = true;
            return false;
        }
        self.next_offset += 1;
        true
    }
}
