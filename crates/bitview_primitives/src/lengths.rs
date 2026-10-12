use brk_types::{Height, OutputType, TxIndex};

use crate::{
    EmptyOutputIndex, OpReturnIndex, P2AAddressIndex, P2MSOutputIndex, P2PK33AddressIndex,
    P2PK65AddressIndex, P2PKHAddressIndex, P2SHAddressIndex, P2TRAddressIndex, P2WPKHAddressIndex,
    P2WSHAddressIndex, TxInIndex, TxOutIndex, TypeIndex, UnknownOutputIndex,
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
    pub p2pk33_addr_index: P2PK33AddressIndex,
    pub p2pk65_addr_index: P2PK65AddressIndex,
    pub p2pkh_addr_index: P2PKHAddressIndex,
    pub p2sh_addr_index: P2SHAddressIndex,
    pub p2tr_addr_index: P2TRAddressIndex,
    pub p2wpkh_addr_index: P2WPKHAddressIndex,
    pub p2wsh_addr_index: P2WSHAddressIndex,
    pub p2a_addr_index: P2AAddressIndex,
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
