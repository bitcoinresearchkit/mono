use crate::{Sats, SatsSigned, TxOut};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

///
/// Address statistics in the mempool (unconfirmed transactions only)
///
/// Based on mempool.space's format.
///
#[derive(Debug, Default, Clone, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct AddrMempoolStats {
    /// Net pending (unconfirmed) balance change in satoshis; negative when pending spends exceed receipts
    pub balance_delta: SatsSigned,

    /// Number of unconfirmed transaction outputs funding this address
    #[cfg_attr(feature = "schemars", schemars(example = 0))]
    pub funded_txo_count: u32,

    /// Total amount in satoshis being received in unconfirmed transactions
    #[cfg_attr(feature = "schemars", schemars(example = Sats::new(0)))]
    pub funded_txo_sum: Sats,

    /// Number of unconfirmed transaction inputs spending from this address
    #[cfg_attr(feature = "schemars", schemars(example = 0))]
    spent_txo_count: u32,

    /// Total amount in satoshis being spent in unconfirmed transactions
    #[cfg_attr(feature = "schemars", schemars(example = Sats::new(0)))]
    pub spent_txo_sum: Sats,

    /// Number of unconfirmed transactions involving this address
    #[cfg_attr(feature = "schemars", schemars(example = 0))]
    pub tx_count: u32,
}

impl AddrMempoolStats {
    pub fn receiving(&mut self, txout: &TxOut) {
        self.balance_delta += SatsSigned::from(txout.value);
        self.funded_txo_count += 1;
        self.funded_txo_sum += txout.value;
    }

    pub fn received(&mut self, txout: &TxOut) {
        self.balance_delta -= SatsSigned::from(txout.value);
        self.funded_txo_count -= 1;
        self.funded_txo_sum -= txout.value;
    }

    pub fn sending(&mut self, txout: &TxOut) {
        self.balance_delta -= SatsSigned::from(txout.value);
        self.spent_txo_count += 1;
        self.spent_txo_sum += txout.value;
    }

    pub fn sent(&mut self, txout: &TxOut) {
        self.balance_delta += SatsSigned::from(txout.value);
        self.spent_txo_count -= 1;
        self.spent_txo_sum -= txout.value;
    }

    pub fn update_tx_count(&mut self, tx_count: u32) {
        self.tx_count = tx_count
    }
}
