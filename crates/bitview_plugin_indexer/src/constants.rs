use std::{str::FromStr, sync::LazyLock};

use bitcoin::Txid as BitcoinTxid;
use brk_types::{Height, TxIndex, Txid, TxidPrefix, Version};

// One version for all data sources
// Increment on **change _OR_ addition**
pub const VERSION: Version = Version::new(42);

/// A BIP30 duplicate coinbase (https://github.com/bitcoin/bips/blob/master/bip-0030.mediawiki): it re-created
/// an earlier coinbase's txid and overwrote that coinbase's unspent outputs, as in Bitcoin Core.
pub struct DuplicateCoinbase {
    txid: &'static str,
    pub height: Height,
    tx_index: TxIndex,
    pub overwritten_height: Height,
    overwritten_tx_index: TxIndex,
}

/// The two duplicate coinbases, both in 2010.
pub const DUPLICATE_COINBASES: [DuplicateCoinbase; 2] = [
    DuplicateCoinbase {
        txid: "d5d27987d2a3dfc724e359870c6644b40e497bdc0589a033220fe15429d88599",
        height: Height::new(91842),
        tx_index: TxIndex::new(142783),
        overwritten_height: Height::new(91812),
        overwritten_tx_index: TxIndex::new(142726),
    },
    DuplicateCoinbase {
        txid: "e3bf3d07d4b0375638d5f1db5255fe07ba2c4cb067cd81b84ee974b6585fb468",
        height: Height::new(91880),
        tx_index: TxIndex::new(142841),
        overwritten_height: Height::new(91722),
        overwritten_tx_index: TxIndex::new(142572),
    },
];

/// The earlier coinbase whose outputs a BIP30 duplicate overwrote.
pub fn overwritten_tx_index(tx_index: TxIndex) -> Option<TxIndex> {
    DUPLICATE_COINBASES
        .iter()
        .find(|dup| dup.tx_index == tx_index)
        .map(|dup| dup.overwritten_tx_index)
}

pub static DUPLICATE_TXIDS: LazyLock<[Txid; 2]> = LazyLock::new(|| {
    DUPLICATE_COINBASES.map(|dup| BitcoinTxid::from_str(dup.txid).unwrap().into())
});

pub static DUPLICATE_TXID_PREFIXES: LazyLock<[(TxidPrefix, TxIndex); 2]> = LazyLock::new(|| {
    DUPLICATE_COINBASES.map(|dup| {
        let txid = Txid::from(BitcoinTxid::from_str(dup.txid).unwrap());
        (TxidPrefix::from(&txid), dup.tx_index)
    })
});
