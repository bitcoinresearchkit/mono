mod address;

use bitview_plugin_indexer::Indexer;
use bitview_primitives::{
    EmptyOutputIndex, OpReturnIndex, P2AAddressIndex, P2MSOutputIndex, P2PK33AddressIndex,
    P2PK65AddressIndex, P2PKHAddressIndex, P2SHAddressIndex, P2TRAddressIndex, P2WPKHAddressIndex,
    P2WSHAddressIndex, UnknownOutputIndex,
};
use bitview_traversable::Traversable;
use brk_types::{
    P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes, P2TRBytes, P2WPKHBytes, P2WSHBytes,
    TxIndex, Version,
};

use self::address::AddressVecs;
use crate::identity::IdentityVecs;

/// Address types: each address index and its textual address.
#[derive(Clone, Traversable)]
pub struct Vecs {
    /// P2PK-shaped outputs containing a 33-byte key field; the field is not
    /// required to be a valid compressed public key.
    pub p2pk33: AddressVecs<P2PK33AddressIndex, P2PK33Bytes>,
    /// P2PK-shaped outputs containing a 65-byte key field; the field is not
    /// required to be a valid uncompressed public key.
    pub p2pk65: AddressVecs<P2PK65AddressIndex, P2PK65Bytes>,
    /// Pay-to-public-key-hash outputs.
    pub p2pkh: AddressVecs<P2PKHAddressIndex, P2PKHBytes>,
    /// Pay-to-script-hash outputs.
    pub p2sh: AddressVecs<P2SHAddressIndex, P2SHBytes>,
    /// Pay-to-Taproot outputs.
    pub p2tr: AddressVecs<P2TRAddressIndex, P2TRBytes>,
    /// Version-0 pay-to-witness-public-key-hash outputs.
    pub p2wpkh: AddressVecs<P2WPKHAddressIndex, P2WPKHBytes>,
    /// Version-0 pay-to-witness-script-hash outputs.
    pub p2wsh: AddressVecs<P2WSHAddressIndex, P2WSHBytes>,
    /// Pay-to-Anchor outputs matching `OP_1 PUSHBYTES_2 0x4e73`.
    pub p2a: AddressVecs<P2AAddressIndex, P2ABytes>,
}

/// Output types without an address: each index and the transaction that created it.
#[derive(Clone, Traversable)]
pub struct OutputVecs {
    /// Bare multisig outputs recognized by Bitcoin script parsing.
    /// Zero-based type-specific output index assigned in canonical chain order.
    pub p2ms: IdentityVecs<P2MSOutputIndex, TxIndex>,
    /// Outputs with an empty locking script.
    /// Zero-based type-specific output index assigned in canonical chain order.
    pub empty: IdentityVecs<EmptyOutputIndex, TxIndex>,
    /// Outputs not matching another recognized locking-script type.
    /// Zero-based type-specific output index assigned in canonical chain order.
    pub unknown: IdentityVecs<UnknownOutputIndex, TxIndex>,
    /// Outputs whose locking script begins with `OP_RETURN`.
    /// Zero-based type-specific output index assigned in canonical chain order.
    pub op_return: IdentityVecs<OpReturnIndex, TxIndex>,
}

impl Vecs {
    pub fn new(version: Version, indexer: &Indexer) -> Self {
        let addrs = &indexer.vecs().addrs;
        Self {
            p2pk33: AddressVecs::new("p2pk33", version, &addrs.p2pk33.bytes),
            p2pk65: AddressVecs::new("p2pk65", version, &addrs.p2pk65.bytes),
            p2pkh: AddressVecs::new("p2pkh", version, &addrs.p2pkh.bytes),
            p2sh: AddressVecs::new("p2sh", version, &addrs.p2sh.bytes),
            p2tr: AddressVecs::new("p2tr", version, &addrs.p2tr.bytes),
            p2wpkh: AddressVecs::new("p2wpkh", version, &addrs.p2wpkh.bytes),
            p2wsh: AddressVecs::new("p2wsh", version, &addrs.p2wsh.bytes),
            p2a: AddressVecs::new("p2a", version, &addrs.p2a.bytes),
        }
    }
}

impl OutputVecs {
    pub fn new(version: Version, indexer: &Indexer) -> Self {
        let scripts = &indexer.vecs().scripts;
        Self {
            p2ms: IdentityVecs::new("p2ms_output_index", version, &scripts.p2ms.to_tx_index),
            empty: IdentityVecs::new("empty_output_index", version, &scripts.empty.to_tx_index),
            unknown: IdentityVecs::new(
                "unknown_output_index",
                version,
                &scripts.unknown.to_tx_index,
            ),
            op_return: IdentityVecs::new(
                "op_return_index",
                version,
                &indexer.vecs().op_return.to_tx_index,
            ),
        }
    }
}
