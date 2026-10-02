use std::ops::RangeInclusive;

use brk_error::Result;
use brk_reader::{BlockReceiver, Reader};
use brk_rpc::Client;
use brk_types::{BlockHash, Height};

pub(crate) enum State {
    Empty,
    Rpc {
        client: Client,
        heights: RangeInclusive<u32>,
        prev_hash: Option<BlockHash>,
    },
    Reader {
        receiver: BlockReceiver,
    },
}

impl State {
    pub(crate) fn new_rpc(client: Client, start: Height, end: Height) -> Self {
        Self::Rpc {
            client,
            heights: *start..=*end,
            prev_hash: None,
        }
    }

    pub(crate) fn new_reader(reader: Reader, start: Height, end: Height) -> Result<Self> {
        let receiver = reader.range(start, end)?;
        Ok(State::Reader { receiver })
    }
}
