mod address;
mod identity;

use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_types::{
    Addr, AddrBytes, EmptyOutputIndex, OpReturnIndex, P2AAddrIndex, P2ABytes, P2MSOutputIndex,
    P2PK33AddrIndex, P2PK33Bytes, P2PK65AddrIndex, P2PK65Bytes, P2PKHAddrIndex, P2PKHBytes,
    P2SHAddrIndex, P2SHBytes, P2TRAddrIndex, P2TRBytes, P2WPKHAddrIndex, P2WPKHBytes,
    P2WSHAddrIndex, P2WSHBytes, TxIndex, UnknownOutputIndex, Version,
};
use vecdb::{LazyVec, ReadableCloneableVec};

use self::{address::AddressVecs, identity::IdentityVecs};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// P2PK-shaped outputs containing a 33-byte key field; the field is not
    /// required to be a valid compressed public key.
    pub p2pk33: AddressVecs<P2PK33AddrIndex, P2PK33Bytes>,
    /// P2PK-shaped outputs containing a 65-byte key field; the field is not
    /// required to be a valid uncompressed public key.
    pub p2pk65: AddressVecs<P2PK65AddrIndex, P2PK65Bytes>,
    /// Pay-to-public-key-hash outputs.
    pub p2pkh: AddressVecs<P2PKHAddrIndex, P2PKHBytes>,
    /// Pay-to-script-hash outputs.
    pub p2sh: AddressVecs<P2SHAddrIndex, P2SHBytes>,
    /// Pay-to-Taproot outputs.
    pub p2tr: AddressVecs<P2TRAddrIndex, P2TRBytes>,
    /// Version-0 pay-to-witness-public-key-hash outputs.
    pub p2wpkh: AddressVecs<P2WPKHAddrIndex, P2WPKHBytes>,
    /// Version-0 pay-to-witness-script-hash outputs.
    pub p2wsh: AddressVecs<P2WSHAddrIndex, P2WSHBytes>,
    /// Pay-to-Anchor outputs matching `OP_1 PUSHBYTES_2 0x4e73`.
    pub p2a: AddressVecs<P2AAddrIndex, P2ABytes>,
    /// Bare multisig outputs recognized by Bitcoin script parsing.
    pub p2ms: IdentityVecs<P2MSOutputIndex, TxIndex>,
    /// Outputs with an empty locking script.
    pub empty: IdentityVecs<EmptyOutputIndex, TxIndex>,
    /// Outputs not matching another recognized locking-script type.
    pub unknown: IdentityVecs<UnknownOutputIndex, TxIndex>,
    /// Outputs whose locking script begins with `OP_RETURN`.
    pub op_return: IdentityVecs<OpReturnIndex, TxIndex>,
}

impl Vecs {
    pub fn forced_import(version: Version, indexer: &Indexer) -> Self {
        Self {
            p2pk33: AddressVecs {
                identity: LazyVec::init(
                    "p2pk33_addr_index",
                    version,
                    indexer.vecs().addrs.p2pk33.bytes.read_only_boxed_clone(),
                    |index, _| index,
                ),
                addr: LazyVec::init(
                    "p2pk33_addr",
                    version,
                    indexer.vecs().addrs.p2pk33.bytes.read_only_boxed_clone(),
                    |_, bytes| Addr::try_from(&AddrBytes::from(bytes)).unwrap(),
                ),
            },
            p2pk65: AddressVecs {
                identity: LazyVec::init(
                    "p2pk65_addr_index",
                    version,
                    indexer.vecs().addrs.p2pk65.bytes.read_only_boxed_clone(),
                    |index, _| index,
                ),
                addr: LazyVec::init(
                    "p2pk65_addr",
                    version,
                    indexer.vecs().addrs.p2pk65.bytes.read_only_boxed_clone(),
                    |_, bytes| Addr::try_from(&AddrBytes::from(bytes)).unwrap(),
                ),
            },
            p2pkh: AddressVecs {
                identity: LazyVec::init(
                    "p2pkh_addr_index",
                    version,
                    indexer.vecs().addrs.p2pkh.bytes.read_only_boxed_clone(),
                    |index, _| index,
                ),
                addr: LazyVec::init(
                    "p2pkh_addr",
                    version,
                    indexer.vecs().addrs.p2pkh.bytes.read_only_boxed_clone(),
                    |_, bytes| Addr::try_from(&AddrBytes::from(bytes)).unwrap(),
                ),
            },
            p2sh: AddressVecs {
                identity: LazyVec::init(
                    "p2sh_addr_index",
                    version,
                    indexer.vecs().addrs.p2sh.bytes.read_only_boxed_clone(),
                    |index, _| index,
                ),
                addr: LazyVec::init(
                    "p2sh_addr",
                    version,
                    indexer.vecs().addrs.p2sh.bytes.read_only_boxed_clone(),
                    |_, bytes| Addr::try_from(&AddrBytes::from(bytes)).unwrap(),
                ),
            },
            p2tr: AddressVecs {
                identity: LazyVec::init(
                    "p2tr_addr_index",
                    version,
                    indexer.vecs().addrs.p2tr.bytes.read_only_boxed_clone(),
                    |index, _| index,
                ),
                addr: LazyVec::init(
                    "p2tr_addr",
                    version,
                    indexer.vecs().addrs.p2tr.bytes.read_only_boxed_clone(),
                    |_, bytes| Addr::try_from(&AddrBytes::from(bytes)).unwrap(),
                ),
            },
            p2wpkh: AddressVecs {
                identity: LazyVec::init(
                    "p2wpkh_addr_index",
                    version,
                    indexer.vecs().addrs.p2wpkh.bytes.read_only_boxed_clone(),
                    |index, _| index,
                ),
                addr: LazyVec::init(
                    "p2wpkh_addr",
                    version,
                    indexer.vecs().addrs.p2wpkh.bytes.read_only_boxed_clone(),
                    |_, bytes| Addr::try_from(&AddrBytes::from(bytes)).unwrap(),
                ),
            },
            p2wsh: AddressVecs {
                identity: LazyVec::init(
                    "p2wsh_addr_index",
                    version,
                    indexer.vecs().addrs.p2wsh.bytes.read_only_boxed_clone(),
                    |index, _| index,
                ),
                addr: LazyVec::init(
                    "p2wsh_addr",
                    version,
                    indexer.vecs().addrs.p2wsh.bytes.read_only_boxed_clone(),
                    |_, bytes| Addr::try_from(&AddrBytes::from(bytes)).unwrap(),
                ),
            },
            p2a: AddressVecs {
                identity: LazyVec::init(
                    "p2a_addr_index",
                    version,
                    indexer.vecs().addrs.p2a.bytes.read_only_boxed_clone(),
                    |index, _| index,
                ),
                addr: LazyVec::init(
                    "p2a_addr",
                    version,
                    indexer.vecs().addrs.p2a.bytes.read_only_boxed_clone(),
                    |_, bytes| Addr::try_from(&AddrBytes::from(bytes)).unwrap(),
                ),
            },
            p2ms: IdentityVecs {
                identity: LazyVec::init(
                    "p2ms_output_index",
                    version,
                    indexer
                        .vecs()
                        .scripts
                        .p2ms
                        .to_tx_index
                        .read_only_boxed_clone(),
                    |index, _| index,
                ),
            },
            empty: IdentityVecs {
                identity: LazyVec::init(
                    "empty_output_index",
                    version,
                    indexer
                        .vecs()
                        .scripts
                        .empty
                        .to_tx_index
                        .read_only_boxed_clone(),
                    |index, _| index,
                ),
            },
            unknown: IdentityVecs {
                identity: LazyVec::init(
                    "unknown_output_index",
                    version,
                    indexer
                        .vecs()
                        .scripts
                        .unknown
                        .to_tx_index
                        .read_only_boxed_clone(),
                    |index, _| index,
                ),
            },
            op_return: IdentityVecs {
                identity: LazyVec::init(
                    "op_return_index",
                    version,
                    indexer.vecs().op_return.to_tx_index.read_only_boxed_clone(),
                    |index, _| index,
                ),
            },
        }
    }
}
