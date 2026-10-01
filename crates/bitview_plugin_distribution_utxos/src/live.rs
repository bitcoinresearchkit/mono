use brk_types::{BlockHash, Cents, Version};

use crate::state::UTXOStates;

pub(crate) struct LiveState {
    pub end: usize,
    pub hash: Option<BlockHash>,
    pub version: Version,
    pub utxos: UTXOStates,
    pub prices: Vec<Cents>,
}
