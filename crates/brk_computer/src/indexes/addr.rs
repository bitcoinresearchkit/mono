mod address;
mod identity;

use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::{
    Addr, AddrBytes, EmptyOutputIndex, OpReturnIndex, P2AAddrIndex, P2ABytes, P2MSOutputIndex,
    P2PK33AddrIndex, P2PK33Bytes, P2PK65AddrIndex, P2PK65Bytes, P2PKHAddrIndex, P2PKHBytes,
    P2SHAddrIndex, P2SHBytes, P2TRAddrIndex, P2TRBytes, P2WPKHAddrIndex, P2WPKHBytes,
    P2WSHAddrIndex, P2WSHBytes, TxIndex, UnknownOutputIndex, Version,
};
use vecdb::{LazyVec, ReadableCloneableVec};

use self::{address::AddressVecs, identity::IdentityVecs};

pub type P2PK33Vecs = AddressVecs<P2PK33AddrIndex, P2PK33Bytes>;
pub type P2PK65Vecs = AddressVecs<P2PK65AddrIndex, P2PK65Bytes>;
pub type P2PKHVecs = AddressVecs<P2PKHAddrIndex, P2PKHBytes>;
pub type P2SHVecs = AddressVecs<P2SHAddrIndex, P2SHBytes>;
pub type P2TRVecs = AddressVecs<P2TRAddrIndex, P2TRBytes>;
pub type P2WPKHVecs = AddressVecs<P2WPKHAddrIndex, P2WPKHBytes>;
pub type P2WSHVecs = AddressVecs<P2WSHAddrIndex, P2WSHBytes>;
pub type P2AVecs = AddressVecs<P2AAddrIndex, P2ABytes>;
pub type P2MSVecs = IdentityVecs<P2MSOutputIndex, TxIndex>;
pub type EmptyVecs = IdentityVecs<EmptyOutputIndex, TxIndex>;
pub type UnknownVecs = IdentityVecs<UnknownOutputIndex, TxIndex>;
pub type OpReturnVecs = IdentityVecs<OpReturnIndex, TxIndex>;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// P2PK-shaped outputs containing a 33-byte key field; the field is not
    /// required to be a valid compressed public key.
    pub p2pk33: P2PK33Vecs,
    /// P2PK-shaped outputs containing a 65-byte key field; the field is not
    /// required to be a valid uncompressed public key.
    pub p2pk65: P2PK65Vecs,
    /// Pay-to-public-key-hash outputs.
    pub p2pkh: P2PKHVecs,
    /// Pay-to-script-hash outputs.
    pub p2sh: P2SHVecs,
    /// Pay-to-Taproot outputs.
    pub p2tr: P2TRVecs,
    /// Version-0 pay-to-witness-public-key-hash outputs.
    pub p2wpkh: P2WPKHVecs,
    /// Version-0 pay-to-witness-script-hash outputs.
    pub p2wsh: P2WSHVecs,
    /// Pay-to-Anchor outputs matching `OP_1 PUSHBYTES_2 0x4e73`.
    pub p2a: P2AVecs,
    /// Bare multisig outputs recognized by Bitcoin script parsing.
    pub p2ms: P2MSVecs,
    /// Outputs with an empty locking script.
    pub empty: EmptyVecs,
    /// Outputs not matching another recognized locking-script type.
    pub unknown: UnknownVecs,
    /// Outputs whose locking script begins with `OP_RETURN`.
    pub op_return: OpReturnVecs,
}

impl Vecs {
    pub(crate) fn forced_import(version: Version, indexer: &Indexer) -> Self {
        Self {
            p2pk33: P2PK33Vecs {
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
            p2pk65: P2PK65Vecs {
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
            p2pkh: P2PKHVecs {
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
            p2sh: P2SHVecs {
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
            p2tr: P2TRVecs {
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
            p2wpkh: P2WPKHVecs {
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
            p2wsh: P2WSHVecs {
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
            p2a: P2AVecs {
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
            p2ms: P2MSVecs {
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
            empty: EmptyVecs {
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
            unknown: UnknownVecs {
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
            op_return: OpReturnVecs {
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
