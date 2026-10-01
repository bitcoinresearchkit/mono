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

#[cfg(test)]
mod tests {
    use brk_rpc::Auth;

    use super::*;
    use crate::BlockIterator;

    fn rpc(start: u32, end: u32) -> State {
        let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
        State::new_rpc(client, Height::new(start), Height::new(end))
    }

    #[test]
    fn rpc_heights_preserve_inclusive_and_empty_ranges() {
        for (start, end, expected) in [
            (0, 0, vec![0]),
            (7, 10, vec![7, 8, 9, 10]),
            (5, 4, vec![]),
            (u32::MAX - 1, u32::MAX, vec![u32::MAX - 1, u32::MAX]),
        ] {
            let State::Rpc { mut heights, .. } = rpc(start, end) else {
                unreachable!();
            };
            assert_eq!(heights.by_ref().collect::<Vec<_>>(), expected);
            assert_eq!(heights.next(), None);
            assert_eq!(heights.next(), None);
        }
    }

    #[test]
    fn empty_rpc_iterator_does_not_contact_the_client() {
        let mut blocks = BlockIterator::new(rpc(5, 4));
        assert!(blocks.next().is_none());
        assert!(blocks.next().is_none());
    }
}
