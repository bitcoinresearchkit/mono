use brk_types::{
    AddrState, EmptyAddrData, ExtendedEmptyAddrIndex, FundedAddrData, FundedAddrIndex, OutputType,
    P2AAddrIndex, P2PK33AddrIndex, P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex, P2TRAddrIndex,
    P2WPKHAddrIndex, P2WSHAddrIndex, TypeIndex,
};
use vecdb::{BytesVecReader, OverflowVecReader};

use crate::addr::AddrStateVecs;

/// Cached readers for address indexes and stateful address data.
pub struct AddrReaders {
    p2a: BytesVecReader<P2AAddrIndex, AddrState>,
    p2pk33: BytesVecReader<P2PK33AddrIndex, AddrState>,
    p2pk65: BytesVecReader<P2PK65AddrIndex, AddrState>,
    p2pkh: BytesVecReader<P2PKHAddrIndex, AddrState>,
    p2sh: BytesVecReader<P2SHAddrIndex, AddrState>,
    p2tr: BytesVecReader<P2TRAddrIndex, AddrState>,
    p2wpkh: BytesVecReader<P2WPKHAddrIndex, AddrState>,
    p2wsh: BytesVecReader<P2WSHAddrIndex, AddrState>,
    funded: OverflowVecReader<FundedAddrIndex, FundedAddrData>,
    extended_empty: OverflowVecReader<ExtendedEmptyAddrIndex, EmptyAddrData>,
}

impl AddrReaders {
    pub fn new(state: &AddrStateVecs) -> Self {
        Self {
            p2a: state.p2a.reader(),
            p2pk33: state.p2pk33.reader(),
            p2pk65: state.p2pk65.reader(),
            p2pkh: state.p2pkh.reader(),
            p2sh: state.p2sh.reader(),
            p2tr: state.p2tr.reader(),
            p2wpkh: state.p2wpkh.reader(),
            p2wsh: state.p2wsh.reader(),
            funded: state.funded.reader(),
            extended_empty: state.extended_empty.reader(),
        }
    }

    pub fn state(
        &self,
        vecs: &AddrStateVecs,
        addr_type: OutputType,
        type_index: TypeIndex,
    ) -> AddrState {
        let state = match addr_type {
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
        state.unwrap()
    }

    #[inline]
    pub fn funded_data(&self, vecs: &AddrStateVecs, index: FundedAddrIndex) -> FundedAddrData {
        vecs.funded.get_with_reader(index, &self.funded).unwrap()
    }

    #[inline]
    pub fn extended_empty_data(
        &self,
        vecs: &AddrStateVecs,
        index: ExtendedEmptyAddrIndex,
    ) -> EmptyAddrData {
        vecs.extended_empty
            .get_with_reader(index, &self.extended_empty)
            .unwrap()
    }
}
