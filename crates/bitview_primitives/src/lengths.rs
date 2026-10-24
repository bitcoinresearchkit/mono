use brk_types::{Height, OutputType, TxIndex};

use crate::{
    EmptyScriptOutputIndex, Index, OpReturnIndex, P2AAddressIndex, P2MSOutputIndex,
    P2PK33AddressIndex, P2PK65AddressIndex, P2PKHAddressIndex, P2SHAddressIndex, P2TRAddressIndex,
    P2WPKHAddressIndex, P2WSHAddressIndex, TxInIndex, TxOutIndex, TypeIndex,
    UnknownScriptOutputIndex,
};

/// Pipeline-wide length/count snapshot.
///
/// `lengths.f = N` means positions `0..N` are fully written; readers reject
/// `position >= lengths.f`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Lengths {
    pub empty_script_output_index: EmptyScriptOutputIndex,
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
    pub unknown_script_output_index: UnknownScriptOutputIndex,
}

macro_rules! lengths_addr_dispatch {
    ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
        impl Lengths {
            /// Next index of an address type, `None` for output types without addresses.
            #[inline]
            fn addr_index_mut(&mut self, output_type: OutputType) -> Option<&mut TypeIndex> {
                match output_type {
                    $(OutputType::$variant => Some(&mut *self.$length),)+
                    _ => None,
                }
            }

            fn clamp_addr_indexes_to(&mut self, other: &Self) {
                $(self.$length = self.$length.min(other.$length);)+
            }
        }
    };
}

crate::with_address_storage_types!(lengths_addr_dispatch);

impl Lengths {
    /// Last fully written block, or `None` before genesis.
    #[inline]
    pub fn last_height(self) -> Option<Height> {
        self.height.decremented()
    }

    /// The first height a six-block reorg could replace: what depends only on blocks below it is
    /// settled. `None` before genesis.
    #[inline]
    pub fn reorg_cutoff(self) -> Option<Height> {
        self.last_height()
            .map(|last| Height::from((*last).saturating_sub(5)))
    }

    /// Whether block `height` lies below the reorg window.
    #[inline]
    pub fn is_settled(self, height: Height) -> bool {
        self.reorg_cutoff().is_some_and(|cutoff| height < cutoff)
    }

    /// The length of `index`, `None` for the indexes not counted here (calendar periods, halving
    /// and difficulty epochs).
    pub fn get(&self, index: Index) -> Option<usize> {
        Some(match index {
            Index::Height => self.height.into(),
            Index::TxIndex => self.tx_index.into(),
            Index::TxInIndex => self.txin_index.into(),
            Index::TxOutIndex => self.txout_index.into(),
            Index::EmptyScriptOutputIndex => self.empty_script_output_index.into(),
            Index::OpReturnIndex => self.op_return_index.into(),
            Index::P2MSOutputIndex => self.p2ms_output_index.into(),
            Index::UnknownScriptOutputIndex => self.unknown_script_output_index.into(),
            Index::P2AAddressIndex => self.p2a_addr_index.into(),
            Index::P2PK33AddressIndex => self.p2pk33_addr_index.into(),
            Index::P2PK65AddressIndex => self.p2pk65_addr_index.into(),
            Index::P2PKHAddressIndex => self.p2pkh_addr_index.into(),
            Index::P2SHAddressIndex => self.p2sh_addr_index.into(),
            Index::P2TRAddressIndex => self.p2tr_addr_index.into(),
            Index::P2WPKHAddressIndex => self.p2wpkh_addr_index.into(),
            Index::P2WSHAddressIndex => self.p2wsh_addr_index.into(),
            Index::Minute10
            | Index::Minute30
            | Index::Hour1
            | Index::Hour4
            | Index::Hour12
            | Index::Day1
            | Index::Day3
            | Index::Week1
            | Index::Month1
            | Index::Month3
            | Index::Month6
            | Index::Year1
            | Index::Year10
            | Index::Halving
            | Index::Epoch => return None,
        })
    }

    pub fn to_type_index(mut self, output_type: OutputType) -> TypeIndex {
        match output_type {
            OutputType::EmptyScript => *self.empty_script_output_index,
            OutputType::OpReturn => *self.op_return_index,
            OutputType::P2MS => *self.p2ms_output_index,
            OutputType::UnknownScript => *self.unknown_script_output_index,
            _ => *self.addr_index_mut(output_type).unwrap(),
        }
    }

    pub fn clamp_to(&mut self, other: &Self) {
        self.height = self.height.min(other.height);
        self.tx_index = self.tx_index.min(other.tx_index);
        self.txin_index = self.txin_index.min(other.txin_index);
        self.txout_index = self.txout_index.min(other.txout_index);
        self.empty_script_output_index = self
            .empty_script_output_index
            .min(other.empty_script_output_index);
        self.op_return_index = self.op_return_index.min(other.op_return_index);
        self.p2ms_output_index = self.p2ms_output_index.min(other.p2ms_output_index);
        self.unknown_script_output_index = self
            .unknown_script_output_index
            .min(other.unknown_script_output_index);
        self.clamp_addr_indexes_to(other);
    }

    pub fn add_block(&mut self, tx_count: usize, input_count: usize, output_count: usize) {
        self.tx_index += TxIndex::from(tx_count);
        self.txin_index += TxInIndex::from(input_count);
        self.txout_index += TxOutIndex::from(output_count);
    }

    /// Increments an address-type index and returns its previous value.
    #[inline]
    pub fn increment_addr_index(&mut self, output_type: OutputType) -> TypeIndex {
        self.addr_index_mut(output_type)
            .expect("an address type")
            .copy_then_increment()
    }
}
