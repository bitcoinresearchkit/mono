use bitview_primitives::{
    AddressState, EmptyAddressData, ExtendedEmptyAddressIndex, FundedAddressData,
    FundedAddressIndex, TypeIndex,
};
use brk_types::OutputType;
use vecdb::{BytesVecReader, OverflowVecReader};

use crate::addr::AddrStateVecs;

macro_rules! addr_readers {
    ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
        pub struct AddrReaders {
            $($field: BytesVecReader<$index, AddressState>,)+
            funded: OverflowVecReader<FundedAddressIndex, FundedAddressData>,
            extended_empty: OverflowVecReader<ExtendedEmptyAddressIndex, EmptyAddressData>,
        }

        impl AddrReaders {
            pub fn new(state: &AddrStateVecs) -> Self {
                Self {
                    $($field: state.$field.reader(),)+
                    funded: state.funded.reader(),
                    extended_empty: state.extended_empty.reader(),
                }
            }

            pub fn state(
                &self,
                vecs: &AddrStateVecs,
                addr_type: OutputType,
                type_index: TypeIndex,
            ) -> AddressState {
                let state = match addr_type {
                    $(OutputType::$variant => {
                        vecs.$field.get_with_reader(type_index.into(), &self.$field)
                    })+
                    _ => unreachable!("invalid address type: {addr_type:?}"),
                };
                state.unwrap()
            }
        }
    };
}

bitview_primitives::with_address_storage_types!(addr_readers);

impl AddrReaders {
    #[inline]
    pub fn funded_data(
        &self,
        vecs: &AddrStateVecs,
        index: FundedAddressIndex,
    ) -> FundedAddressData {
        vecs.funded.get_with_reader(index, &self.funded).unwrap()
    }

    #[inline]
    pub fn extended_empty_data(
        &self,
        vecs: &AddrStateVecs,
        index: ExtendedEmptyAddressIndex,
    ) -> EmptyAddressData {
        vecs.extended_empty
            .get_with_reader(index, &self.extended_empty)
            .unwrap()
    }
}
