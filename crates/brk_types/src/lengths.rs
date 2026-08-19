use crate::{
    EmptyOutputIndex, Height, OpReturnIndex, OutputType, P2AAddrIndex, P2MSOutputIndex,
    P2PK33AddrIndex, P2PK65AddrIndex, P2PKHAddrIndex, P2SHAddrIndex, P2TRAddrIndex,
    P2WPKHAddrIndex, P2WSHAddrIndex, TxInIndex, TxIndex, TxOutIndex, TypeIndex, UnknownOutputIndex,
};

/// Pipeline-wide length/count snapshot.
///
/// `lengths.f = N` means positions `0..N` are fully written; readers reject
/// `position >= lengths.f`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Lengths {
    pub empty_output_index: EmptyOutputIndex,
    pub height: Height,
    pub op_return_index: OpReturnIndex,
    pub p2ms_output_index: P2MSOutputIndex,
    pub p2pk33_addr_index: P2PK33AddrIndex,
    pub p2pk65_addr_index: P2PK65AddrIndex,
    pub p2pkh_addr_index: P2PKHAddrIndex,
    pub p2sh_addr_index: P2SHAddrIndex,
    pub p2tr_addr_index: P2TRAddrIndex,
    pub p2wpkh_addr_index: P2WPKHAddrIndex,
    pub p2wsh_addr_index: P2WSHAddrIndex,
    pub p2a_addr_index: P2AAddrIndex,
    pub tx_index: TxIndex,
    pub txin_index: TxInIndex,
    pub txout_index: TxOutIndex,
    pub unknown_output_index: UnknownOutputIndex,
}

impl Lengths {
    /// Last fully written block, or `None` before genesis.
    #[inline]
    pub fn last_height(self) -> Option<Height> {
        self.height.decremented()
    }

    pub fn to_type_index(self, output_type: OutputType) -> TypeIndex {
        match output_type {
            OutputType::Empty => *self.empty_output_index,
            OutputType::OpReturn => *self.op_return_index,
            OutputType::P2A => *self.p2a_addr_index,
            OutputType::P2MS => *self.p2ms_output_index,
            OutputType::P2PK33 => *self.p2pk33_addr_index,
            OutputType::P2PK65 => *self.p2pk65_addr_index,
            OutputType::P2PKH => *self.p2pkh_addr_index,
            OutputType::P2SH => *self.p2sh_addr_index,
            OutputType::P2TR => *self.p2tr_addr_index,
            OutputType::P2WPKH => *self.p2wpkh_addr_index,
            OutputType::P2WSH => *self.p2wsh_addr_index,
            OutputType::Unknown => *self.unknown_output_index,
        }
    }

    pub fn clamp_to(&mut self, other: &Self) {
        self.height = self.height.min(other.height);
        self.tx_index = self.tx_index.min(other.tx_index);
        self.txin_index = self.txin_index.min(other.txin_index);
        self.txout_index = self.txout_index.min(other.txout_index);
        self.empty_output_index = self.empty_output_index.min(other.empty_output_index);
        self.op_return_index = self.op_return_index.min(other.op_return_index);
        self.p2ms_output_index = self.p2ms_output_index.min(other.p2ms_output_index);
        self.p2pk33_addr_index = self.p2pk33_addr_index.min(other.p2pk33_addr_index);
        self.p2pk65_addr_index = self.p2pk65_addr_index.min(other.p2pk65_addr_index);
        self.p2pkh_addr_index = self.p2pkh_addr_index.min(other.p2pkh_addr_index);
        self.p2sh_addr_index = self.p2sh_addr_index.min(other.p2sh_addr_index);
        self.p2tr_addr_index = self.p2tr_addr_index.min(other.p2tr_addr_index);
        self.p2wpkh_addr_index = self.p2wpkh_addr_index.min(other.p2wpkh_addr_index);
        self.p2wsh_addr_index = self.p2wsh_addr_index.min(other.p2wsh_addr_index);
        self.p2a_addr_index = self.p2a_addr_index.min(other.p2a_addr_index);
        self.unknown_output_index = self.unknown_output_index.min(other.unknown_output_index);
    }

    /// Bumps the entity totals after processing a block.
    pub fn add_block(&mut self, tx_count: usize, input_count: usize, output_count: usize) {
        self.tx_index += TxIndex::from(tx_count);
        self.txin_index += TxInIndex::from(input_count);
        self.txout_index += TxOutIndex::from(output_count);
    }

    /// Increments an address-type index and returns its previous value.
    #[inline]
    pub fn increment_addr_index(&mut self, output_type: OutputType) -> TypeIndex {
        match output_type {
            OutputType::P2PK65 => self.p2pk65_addr_index.copy_then_increment(),
            OutputType::P2PK33 => self.p2pk33_addr_index.copy_then_increment(),
            OutputType::P2PKH => self.p2pkh_addr_index.copy_then_increment(),
            OutputType::P2SH => self.p2sh_addr_index.copy_then_increment(),
            OutputType::P2WPKH => self.p2wpkh_addr_index.copy_then_increment(),
            OutputType::P2WSH => self.p2wsh_addr_index.copy_then_increment(),
            OutputType::P2TR => self.p2tr_addr_index.copy_then_increment(),
            OutputType::P2A => self.p2a_addr_index.copy_then_increment(),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_is_componentwise() {
        let value = u32::MAX as usize;
        let mut lengths = Lengths {
            empty_output_index: EmptyOutputIndex::from(value),
            height: Height::from(value),
            op_return_index: OpReturnIndex::from(value),
            p2ms_output_index: P2MSOutputIndex::from(value),
            p2pk33_addr_index: P2PK33AddrIndex::from(value),
            p2pk65_addr_index: P2PK65AddrIndex::from(value),
            p2pkh_addr_index: P2PKHAddrIndex::from(value),
            p2sh_addr_index: P2SHAddrIndex::from(value),
            p2tr_addr_index: P2TRAddrIndex::from(value),
            p2wpkh_addr_index: P2WPKHAddrIndex::from(value),
            p2wsh_addr_index: P2WSHAddrIndex::from(value),
            p2a_addr_index: P2AAddrIndex::from(value),
            tx_index: TxIndex::from(value),
            txin_index: TxInIndex::from(value),
            txout_index: TxOutIndex::from(value),
            unknown_output_index: UnknownOutputIndex::from(value),
        };
        let minimum = Lengths::default();

        lengths.clamp_to(&minimum);

        assert_eq!(lengths, minimum);
    }

    #[test]
    fn last_height_converts_length_to_position() {
        assert_eq!(Lengths::default().last_height(), None);
        assert_eq!(
            Lengths {
                height: Height::from(2_u32),
                ..Default::default()
            }
            .last_height(),
            Some(Height::from(1_u32))
        );
    }
}
