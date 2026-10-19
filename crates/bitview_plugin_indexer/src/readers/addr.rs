use bitview_primitives::TypeIndex;
use brk_types::{AddressBytes, OutputType};
use vecdb::{BytesVecReader, Ro, Rw};

use crate::AddrsVecs;

macro_rules! addr_readers {
    ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
        pub struct AddrReaders {
            $(pub $field: BytesVecReader<$index, $bytes>,)+
        }

        impl AddrReaders {
            #[inline(always)]
            pub fn get(&self, output_type: OutputType, type_index: TypeIndex) -> Option<AddressBytes> {
                let index = usize::from(type_index);
                match output_type {
                    $(OutputType::$variant => self.$field.try_get_at(index).map(Into::into),)+
                    _ => None,
                }
            }
        }

        impl AddrsVecs<Rw> {
            pub fn addr_readers(&self) -> AddrReaders {
                AddrReaders {
                    $($field: self.$field.bytes.reader(),)+
                }
            }
        }

        impl AddrsVecs<Ro> {
            pub fn addr_readers(&self) -> AddrReaders {
                AddrReaders {
                    $($field: self.$field.bytes.reader(),)+
                }
            }
        }
    };
}

bitview_primitives::with_address_storage_types!(addr_readers);
