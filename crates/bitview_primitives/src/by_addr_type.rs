use brk_types::OutputType;

#[cfg(feature = "storage")]
use rayon::prelude::*;

/// Calls `$callback! { (Variant, field, lengths_field, IndexType, BytesType), ... }` with the
/// eight address types the indexer stores, in storage order: the `OutputType`/`AddrTypeId`
/// variant, the field name of per-type collections, the `Lengths` field, the typed index and
/// the address bytes. P2PK keeps its 33- and 65-byte keys apart and P2A counts as an address.
#[macro_export]
macro_rules! with_address_storage_types {
    ($callback:ident) => {
        $callback! {
            (P2PK65, p2pk65, p2pk65_addr_index, $crate::P2PK65AddressIndex, ::brk_types::P2PK65Bytes),
            (P2PK33, p2pk33, p2pk33_addr_index, $crate::P2PK33AddressIndex, ::brk_types::P2PK33Bytes),
            (P2PKH, p2pkh, p2pkh_addr_index, $crate::P2PKHAddressIndex, ::brk_types::P2PKHBytes),
            (P2SH, p2sh, p2sh_addr_index, $crate::P2SHAddressIndex, ::brk_types::P2SHBytes),
            (P2WPKH, p2wpkh, p2wpkh_addr_index, $crate::P2WPKHAddressIndex, ::brk_types::P2WPKHBytes),
            (P2WSH, p2wsh, p2wsh_addr_index, $crate::P2WSHAddressIndex, ::brk_types::P2WSHBytes),
            (P2TR, p2tr, p2tr_addr_index, $crate::P2TRAddressIndex, ::brk_types::P2TRBytes),
            (P2A, p2a, p2a_addr_index, $crate::P2AAddressIndex, ::brk_types::P2ABytes),
        }
    };
}

macro_rules! by_addr_type {
    ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
        /// One of the eight address types the indexer stores: a physical storage slot, not an
        /// analytics cohort.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum AddrTypeId {
            $($variant),+
        }

        impl AddrTypeId {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($field)),+
                }
            }

            pub const fn output_type(self) -> OutputType {
                match self {
                    $(Self::$variant => OutputType::$variant),+
                }
            }

            #[inline]
            pub fn select<T>(self, values: &ByAddrType<T>) -> &T {
                match self {
                    $(Self::$variant => &values.$field),+
                }
            }

            #[inline]
            pub fn select_mut<T>(self, values: &mut ByAddrType<T>) -> &mut T {
                match self {
                    $(Self::$variant => &mut values.$field),+
                }
            }
        }

        /// One value per stored address type.
        #[derive(Default, Clone, Debug)]
        pub struct ByAddrType<T> {
            $(pub $field: T),+
        }

        impl<T> ByAddrType<T> {
            pub fn from_fn(mut create: impl FnMut(AddrTypeId) -> T) -> Self {
                Self {
                    $($field: create(AddrTypeId::$variant)),+
                }
            }

            pub fn try_from_fn<E>(
                mut create: impl FnMut(AddrTypeId) -> Result<T, E>,
            ) -> Result<Self, E> {
                Ok(Self {
                    $($field: create(AddrTypeId::$variant)?),+
                })
            }

            #[inline]
            pub fn get(&self, addr_type: OutputType) -> Option<&T> {
                match addr_type {
                    $(OutputType::$variant => Some(&self.$field),)+
                    _ => None,
                }
            }

            #[inline]
            pub fn get_mut(&mut self, addr_type: OutputType) -> Option<&mut T> {
                match addr_type {
                    $(OutputType::$variant => Some(&mut self.$field),)+
                    _ => None,
                }
            }

            #[inline]
            fn as_array(&self) -> [&T; AddrTypeId::ALL.len()] {
                [$(&self.$field),+]
            }

            #[inline]
            fn as_array_mut(&mut self) -> [&mut T; AddrTypeId::ALL.len()] {
                [$(&mut self.$field),+]
            }

            #[inline]
            #[allow(clippy::should_implement_trait)]
            pub fn into_iter(self) -> impl Iterator<Item = (OutputType, T)> {
                [$((OutputType::$variant, self.$field)),+].into_iter()
            }
        }
    };
}

with_address_storage_types!(by_addr_type);

impl<T> ByAddrType<T> {
    #[cfg(feature = "storage")]
    pub fn par_try_from_fn<E>(
        create: impl Fn(AddrTypeId) -> Result<T, E> + Send + Sync,
    ) -> Result<Self, E>
    where
        T: Send,
        E: Send,
    {
        let mut values = AddrTypeId::ALL
            .par_iter()
            .copied()
            .map(create)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter();
        Ok(Self::from_fn(|_| {
            values.next().expect("one value per address type")
        }))
    }

    #[inline]
    pub fn get_unwrap(&self, addr_type: OutputType) -> &T {
        self.get(addr_type).unwrap()
    }

    #[inline]
    pub fn get_mut_unwrap(&mut self, addr_type: OutputType) -> &mut T {
        self.get_mut(addr_type).unwrap()
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.as_array().into_iter()
    }

    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.as_array_mut().into_iter()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (OutputType, &T)> {
        AddrTypeId::ALL
            .iter()
            .map(|id| id.output_type())
            .zip(self.values())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (OutputType, &mut T)> {
        AddrTypeId::ALL
            .iter()
            .map(|id| id.output_type())
            .zip(self.values_mut())
    }

    #[inline]
    pub fn output_type_refs(&self) -> [Option<&T>; OutputType::COUNT] {
        let mut refs = [None; OutputType::COUNT];
        for (output_type, value) in self.iter() {
            refs[output_type as usize] = Some(value);
        }
        refs
    }
}
