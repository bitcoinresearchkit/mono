use std::str::FromStr;

use brk_types::{TxIndex, Txid, TxidPrefix, Version};

// One version for all data sources
// Increment on **change _OR_ addition**
pub const VERSION: Version = Version::new(34);

/// Known duplicate Bitcoin transactions (BIP30)
/// https://github.com/bitcoin/bips/blob/master/bip-0030.mediawiki
/// Each entry is (txid_str, tx_index) - these are coinbase txs that were duplicated.
pub const DUPLICATE_TXID_STRS: [(&str, u32); 2] = [
    (
        "d5d27987d2a3dfc724e359870c6644b40e497bdc0589a033220fe15429d88599",
        142783,
    ),
    (
        "e3bf3d07d4b0375638d5f1db5255fe07ba2c4cb067cd81b84ee974b6585fb468",
        142841,
    ),
];

pub static DUPLICATE_TXIDS: std::sync::LazyLock<[Txid; 2]> = std::sync::LazyLock::new(|| {
    [
        bitcoin::Txid::from_str(DUPLICATE_TXID_STRS[0].0)
            .unwrap()
            .into(),
        bitcoin::Txid::from_str(DUPLICATE_TXID_STRS[1].0)
            .unwrap()
            .into(),
    ]
});

pub static DUPLICATE_TXID_PREFIXES: std::sync::LazyLock<[(TxidPrefix, TxIndex); 2]> =
    std::sync::LazyLock::new(|| {
        DUPLICATE_TXID_STRS.map(|(s, tx_index)| {
            (
                TxidPrefix::from(&Txid::from(bitcoin::Txid::from_str(s).unwrap())),
                TxIndex::new(tx_index),
            )
        })
    });
