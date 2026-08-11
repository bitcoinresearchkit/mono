use brk_types::{
    AnyAddrIndex, EmptyAddrData, EmptyAddrIndex, FundedAddrData, FundedAddrIndex, OutputType,
    P2AAddrIndex, P2PK33AddrIndex, P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex, P2TRAddrIndex,
    P2WPKHAddrIndex, P2WSHAddrIndex, TypeIndex,
};
use vecdb::BytesVecReader;

use crate::distribution::addr::{AddrsDataVecs, AnyAddrIndexesVecs};

/// Cached readers for address indexes and stateful address data.
pub struct AddrReaders {
    p2a: BytesVecReader<P2AAddrIndex, AnyAddrIndex>,
    p2pk33: BytesVecReader<P2PK33AddrIndex, AnyAddrIndex>,
    p2pk65: BytesVecReader<P2PK65AddrIndex, AnyAddrIndex>,
    p2pkh: BytesVecReader<P2PKHAddrIndex, AnyAddrIndex>,
    p2sh: BytesVecReader<P2SHAddrIndex, AnyAddrIndex>,
    p2tr: BytesVecReader<P2TRAddrIndex, AnyAddrIndex>,
    p2wpkh: BytesVecReader<P2WPKHAddrIndex, AnyAddrIndex>,
    p2wsh: BytesVecReader<P2WSHAddrIndex, AnyAddrIndex>,
    funded: BytesVecReader<FundedAddrIndex, FundedAddrData>,
    empty: BytesVecReader<EmptyAddrIndex, EmptyAddrData>,
}

impl AddrReaders {
    pub(crate) fn new(any_addr_indexes: &AnyAddrIndexesVecs, addrs_data: &AddrsDataVecs) -> Self {
        Self {
            p2a: any_addr_indexes.p2a.reader(),
            p2pk33: any_addr_indexes.p2pk33.reader(),
            p2pk65: any_addr_indexes.p2pk65.reader(),
            p2pkh: any_addr_indexes.p2pkh.reader(),
            p2sh: any_addr_indexes.p2sh.reader(),
            p2tr: any_addr_indexes.p2tr.reader(),
            p2wpkh: any_addr_indexes.p2wpkh.reader(),
            p2wsh: any_addr_indexes.p2wsh.reader(),
            funded: addrs_data.funded.reader(),
            empty: addrs_data.empty.reader(),
        }
    }

    pub(crate) fn any_addr_index(
        &self,
        vecs: &AnyAddrIndexesVecs,
        addr_type: OutputType,
        type_index: TypeIndex,
    ) -> AnyAddrIndex {
        let index = match addr_type {
            OutputType::P2A => vecs.p2a.get_with_reader(type_index.into(), &self.p2a),
            OutputType::P2PK33 => vecs.p2pk33.get_with_reader(type_index.into(), &self.p2pk33),
            OutputType::P2PK65 => vecs.p2pk65.get_with_reader(type_index.into(), &self.p2pk65),
            OutputType::P2PKH => vecs.p2pkh.get_with_reader(type_index.into(), &self.p2pkh),
            OutputType::P2SH => vecs.p2sh.get_with_reader(type_index.into(), &self.p2sh),
            OutputType::P2TR => vecs.p2tr.get_with_reader(type_index.into(), &self.p2tr),
            OutputType::P2WPKH => vecs.p2wpkh.get_with_reader(type_index.into(), &self.p2wpkh),
            OutputType::P2WSH => vecs.p2wsh.get_with_reader(type_index.into(), &self.p2wsh),
            _ => unreachable!("invalid address type: {addr_type:?}"),
        };
        index.unwrap()
    }

    #[inline]
    pub(crate) fn funded_data(
        &self,
        vecs: &AddrsDataVecs,
        index: FundedAddrIndex,
    ) -> FundedAddrData {
        vecs.funded.get_with_reader(index, &self.funded).unwrap()
    }

    #[inline]
    pub(crate) fn empty_data(&self, vecs: &AddrsDataVecs, index: EmptyAddrIndex) -> EmptyAddrData {
        vecs.empty.get_with_reader(index, &self.empty).unwrap()
    }
}
