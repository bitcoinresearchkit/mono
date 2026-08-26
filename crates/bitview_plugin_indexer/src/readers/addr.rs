use bitcoin::ScriptBuf;
use brk_types::{
    AddrBytes, OutputType, P2AAddrIndex, P2ABytes, P2PK33AddrIndex, P2PK33Bytes, P2PK65AddrIndex,
    P2PK65Bytes, P2PKHAddrIndex, P2PKHBytes, P2SHAddrIndex, P2SHBytes, P2TRAddrIndex, P2TRBytes,
    P2WPKHAddrIndex, P2WPKHBytes, P2WSHAddrIndex, P2WSHBytes, TypeIndex,
};
use vecdb::BytesVecReader;

pub struct AddrReaders {
    pub p2pk65: BytesVecReader<P2PK65AddrIndex, P2PK65Bytes>,
    pub p2pk33: BytesVecReader<P2PK33AddrIndex, P2PK33Bytes>,
    pub p2pkh: BytesVecReader<P2PKHAddrIndex, P2PKHBytes>,
    pub p2sh: BytesVecReader<P2SHAddrIndex, P2SHBytes>,
    pub p2wpkh: BytesVecReader<P2WPKHAddrIndex, P2WPKHBytes>,
    pub p2wsh: BytesVecReader<P2WSHAddrIndex, P2WSHBytes>,
    pub p2tr: BytesVecReader<P2TRAddrIndex, P2TRBytes>,
    pub p2a: BytesVecReader<P2AAddrIndex, P2ABytes>,
}

impl AddrReaders {
    #[inline(always)]
    pub fn get(&self, output_type: OutputType, type_index: TypeIndex) -> Option<AddrBytes> {
        let index = usize::from(type_index);
        match output_type {
            OutputType::P2PK65 => self.p2pk65.try_get_at(index).map(Into::into),
            OutputType::P2PK33 => self.p2pk33.try_get_at(index).map(Into::into),
            OutputType::P2PKH => self.p2pkh.try_get_at(index).map(Into::into),
            OutputType::P2SH => self.p2sh.try_get_at(index).map(Into::into),
            OutputType::P2WPKH => self.p2wpkh.try_get_at(index).map(Into::into),
            OutputType::P2WSH => self.p2wsh.try_get_at(index).map(Into::into),
            OutputType::P2TR => self.p2tr.try_get_at(index).map(Into::into),
            OutputType::P2A => self.p2a.try_get_at(index).map(Into::into),
            _ => None,
        }
    }

    pub fn script_pubkey(&self, output_type: OutputType, type_index: TypeIndex) -> ScriptBuf {
        self.get(output_type, type_index)
            .map(|bytes| bytes.to_script_pubkey())
            .unwrap_or_default()
    }
}
