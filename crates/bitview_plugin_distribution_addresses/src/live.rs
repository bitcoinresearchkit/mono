use brk_types::{BlockHash, Cents, Version};

use crate::state::AddrStates;

pub(crate) struct LiveState {
    pub end: usize,
    pub hash: Option<BlockHash>,
    pub version: Version,
    pub addrs: AddrStates,
    pub prices: Vec<Cents>,
}
