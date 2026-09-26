use std::path::Path;

use brk_types::Height;

use super::{AddrStates, UTXOStates};

/// Writer-owned cohort state at the end of a successful update.
pub struct LiveState {
    pub height: Height,
    pub utxos: UTXOStates,
    pub addrs: AddrStates,
}

impl LiveState {
    pub fn new(path: &Path) -> Self {
        Self {
            height: Height::ZERO,
            utxos: UTXOStates::new(path),
            addrs: AddrStates::new(path),
        }
    }
}
