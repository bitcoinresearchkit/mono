use bitview_primitives::SupplyState;
use brk_types::{Height, Sats, TxIndex, Version};

// One schema version for every data source: bump it when stored data changes or a source is
// added.
pub const VERSION: Version = Version::new(45);

/// A BIP30 duplicate coinbase (https://github.com/bitcoin/bips/blob/master/bip-0030.mediawiki): it re-created
/// an earlier coinbase's txid and overwrote that coinbase's unspent outputs, as in Bitcoin Core.
/// The txid store keeps mapping the shared txid to the original.
struct DuplicateCoinbase {
    height: Height,
    tx_index: TxIndex,
    overwritten_height: Height,
    overwritten_tx_index: TxIndex,
}

/// The two duplicate coinbases, both in 2010.
const DUPLICATE_COINBASES: [DuplicateCoinbase; 2] = [
    // txid d5d27987d2a3dfc724e359870c6644b40e497bdc0589a033220fe15429d88599
    DuplicateCoinbase {
        height: Height::new(91842),
        tx_index: TxIndex::new(142783),
        overwritten_height: Height::new(91812),
        overwritten_tx_index: TxIndex::new(142726),
    },
    // txid e3bf3d07d4b0375638d5f1db5255fe07ba2c4cb067cd81b84ee974b6585fb468
    DuplicateCoinbase {
        height: Height::new(91880),
        tx_index: TxIndex::new(142841),
        overwritten_height: Height::new(91722),
        overwritten_tx_index: TxIndex::new(142572),
    },
];

/// The output a BIP30 duplicate coinbase at `height` overwrote: the earlier coinbase's height and
/// its one lost 50 BTC output.
pub fn overwritten_output(height: Height) -> Option<(Height, SupplyState)> {
    let dup = DUPLICATE_COINBASES
        .iter()
        .find(|dup| dup.height == height)?;
    Some((
        dup.overwritten_height,
        SupplyState {
            value: Sats::FIFTY_BTC,
            utxo_count: 1,
        },
    ))
}

/// The earlier coinbase whose outputs a BIP30 duplicate overwrote.
pub fn overwritten_tx_index(tx_index: TxIndex) -> Option<TxIndex> {
    DUPLICATE_COINBASES
        .iter()
        .find(|dup| dup.tx_index == tx_index)
        .map(|dup| dup.overwritten_tx_index)
}
