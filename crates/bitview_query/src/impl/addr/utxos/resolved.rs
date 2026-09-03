use brk_types::{AddrBytes, BlockHash, Height, OutputType, TypeIndex};

/// An address UTXO query bound to one best-chain view.
pub struct ResolvedAddrUtxos {
    addr: AddrBytes,
    output_type: OutputType,
    type_index: TypeIndex,
    anchor: (Height, BlockHash),
    tip: BlockHash,
}

impl ResolvedAddrUtxos {
    pub(super) fn new(
        addr: AddrBytes,
        output_type: OutputType,
        type_index: TypeIndex,
        anchor: (Height, BlockHash),
        tip: BlockHash,
    ) -> Self {
        Self {
            addr,
            output_type,
            type_index,
            anchor,
            tip,
        }
    }

    pub fn block_hash(&self) -> BlockHash {
        self.anchor.1
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        AddrBytes,
        OutputType,
        TypeIndex,
        (Height, BlockHash),
        BlockHash,
    ) {
        (
            self.addr,
            self.output_type,
            self.type_index,
            self.anchor,
            self.tip,
        )
    }
}
