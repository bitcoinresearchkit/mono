use crate::{Sats, SatsSigned, TxOut};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Address statistics in the mempool (unconfirmed transactions only), in mempool.space's format.
#[derive(Debug, Default, Clone, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct AddressMempoolStats {
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

impl AddressMempoolStats {
    pub fn add_funding(&mut self, txout: &TxOut) {
        self.balance_delta += SatsSigned::from(txout.value);
        self.funded_txo_count += 1;
        self.funded_txo_sum += txout.value;
    }

    pub fn remove_funding(&mut self, txout: &TxOut) {
        self.balance_delta -= SatsSigned::from(txout.value);
        self.funded_txo_count -= 1;
        self.funded_txo_sum -= txout.value;
    }

    pub fn add_spend(&mut self, txout: &TxOut) {
        self.balance_delta -= SatsSigned::from(txout.value);
        self.spent_txo_count += 1;
        self.spent_txo_sum += txout.value;
    }

    pub fn remove_spend(&mut self, txout: &TxOut) {
        self.balance_delta += SatsSigned::from(txout.value);
        self.spent_txo_count -= 1;
        self.spent_txo_sum -= txout.value;
    }
}
