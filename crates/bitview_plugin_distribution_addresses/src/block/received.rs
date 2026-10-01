use brk_types::{Sats, TxIndex};

use super::TxIndexes;

/// One address's received outputs and transactions within a block.
#[derive(Debug)]
pub struct Received {
    pub total_value: Sats,
    pub output_count: u32,
    pub tx_indexes: TxIndexes,
}

impl Received {
    pub fn new(value: Sats, tx_index: TxIndex) -> Self {
        Self {
            total_value: value,
            output_count: 1,
            tx_indexes: TxIndexes::new(tx_index),
        }
    }

    pub fn add(&mut self, value: Sats, tx_index: TxIndex) {
        self.total_value += value;
        self.output_count += 1;
        self.tx_indexes.push(tx_index);
    }
}
