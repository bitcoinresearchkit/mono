use std::{
    fmt::{Display, Formatter, Result},
    ops::Add,
};

use brk_types::CheckedSub;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco, PrintableIndex, VecIndex};

/// An index within its type (0 for the first P2WPKH address).
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    Hash,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct TypeIndex(u32);

impl TypeIndex {
    pub const COINBASE: Self = Self(u32::MAX);

    pub fn new(i: u32) -> Self {
        Self(i)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn copy_then_increment(&mut self) -> Self {
        let i = *self;
        self.increment();
        i
    }
}

impl From<u32> for TypeIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<TypeIndex> for u32 {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        value.0
    }
}

impl From<u64> for TypeIndex {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value as u32)
    }
}
impl From<TypeIndex> for u64 {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        value.0 as u64
    }
}

impl From<usize> for TypeIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}
impl From<TypeIndex> for usize {
    #[inline]
    fn from(value: TypeIndex) -> Self {
        value.0 as usize
    }
}

impl Add<u32> for TypeIndex {
    type Output = Self;
    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl Add<usize> for TypeIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as u32)
    }
}

impl Add<TypeIndex> for TypeIndex {
    type Output = Self;
    fn add(self, rhs: TypeIndex) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

brk_types::impl_store_value!(TypeIndex, u32);

impl CheckedSub<TypeIndex> for TypeIndex {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

brk_types::impl_int_format!(TypeIndex);

/// A [`TypeIndex`] newtype for one script type's entities, with its `Index` name, aliases and
/// initial vec capacity.
macro_rules! typed_index {
    ($(#[$meta:meta])* $name:ident, $index_name:literal, [$($alias:literal),+], $capacity:expr) => {
        $(#[$meta])*
        #[derive(
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Clone,
            Copy,
            Deref,
            DerefMut,
            Default,
            Serialize,
            Deserialize,
            JsonSchema,
        )]
        #[cfg_attr(feature = "storage", derive(Pco))]
        pub struct $name(TypeIndex);

        impl From<TypeIndex> for $name {
            #[inline]
            fn from(value: TypeIndex) -> Self {
                Self(value)
            }
        }

        impl From<$name> for TypeIndex {
            #[inline]
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<u32> for $name {
            #[inline]
            fn from(value: u32) -> Self {
                Self(TypeIndex::from(value))
            }
        }

        impl From<$name> for u32 {
            #[inline]
            fn from(value: $name) -> Self {
                Self::from(value.0)
            }
        }

        impl From<usize> for $name {
            #[inline]
            fn from(value: usize) -> Self {
                Self(TypeIndex::from(value))
            }
        }

        impl From<$name> for usize {
            #[inline]
            fn from(value: $name) -> Self {
                Self::from(value.0)
            }
        }

        impl From<$name> for u64 {
            #[inline]
            fn from(value: $name) -> Self {
                Self::from(value.0)
            }
        }

        impl Add<usize> for $name {
            type Output = Self;
            #[inline]
            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 + rhs)
            }
        }

        impl CheckedSub for $name {
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                CheckedSub::checked_sub(self.0, rhs.0).map(Self)
            }
        }

        impl $name {
            pub(crate) fn index_name() -> &'static str {
                $index_name
            }
            pub(crate) fn index_aliases() -> &'static [&'static str] {
                &[$($alias),+]
            }
        }

        #[cfg(feature = "storage")]
        impl PrintableIndex for $name {
            fn to_string() -> &'static str {
                Self::index_name()
            }
        }

        #[cfg(feature = "storage")]
        impl VecIndex for $name {
            const INITIAL_CAPACITY: usize = $capacity;
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                self.0.fmt(f)
            }
        }

        #[cfg(feature = "storage")]
        impl Formattable for $name {
            #[inline(always)]
            fn write_to(&self, buf: &mut Vec<u8>) {
                self.0.write_to(buf);
            }
        }
    };
}

typed_index!(
    /// Index of a P2PK address with an uncompressed (65-byte) public key.
    P2PK65AddressIndex, "p2pk65_address_index", ["pk65addr", "p2pk65addr", "p2pk65_address_index"], 250_000
);
typed_index!(
    /// Index of a P2PK address with a compressed (33-byte) public key.
    P2PK33AddressIndex, "p2pk33_address_index", ["pk33addr", "p2pk33addr", "p2pk33_address_index"], 40_000
);
typed_index!(
    /// Index of a P2PKH address.
    P2PKHAddressIndex, "p2pkh_address_index", ["pkhaddr", "p2pkhaddr", "p2pkh_address_index"], 750_000_000
);
typed_index!(
    /// Index of a P2SH address.
    P2SHAddressIndex, "p2sh_address_index", ["shaddr", "p2shaddr", "p2sh_address_index"], 500_000_000
);
typed_index!(
    /// Index of a P2WPKH address.
    P2WPKHAddressIndex, "p2wpkh_address_index", ["wpkhaddr", "p2wpkhaddr", "p2wpkh_address_index"], 500_000_000
);
typed_index!(
    /// Index of a P2WSH address.
    P2WSHAddressIndex, "p2wsh_address_index", ["wshaddr", "p2wshaddr", "p2wsh_address_index"], 50_000_000
);
typed_index!(
    /// Index of a P2TR (taproot) address.
    P2TRAddressIndex, "p2tr_address_index", ["traddr", "p2traddr", "p2tr_address_index"], 100_000_000
);
typed_index!(
    /// Index of a P2A (pay-to-anchor) address.
    P2AAddressIndex, "p2a_address_index", ["aaddr", "p2aaddr", "p2a_address_index"], 1_000_000
);
typed_index!(
    /// Index of a P2MS (bare multisig) output.
    P2MSOutputIndex, "p2ms_output_index", ["msout", "p2msout", "p2ms_output_index"], 4_000_000
);
typed_index!(
    /// Index of an OP_RETURN output.
    OpReturnIndex, "op_return_index", ["op", "opreturn", "op_return", "op_return_index"], 0
);
typed_index!(
    /// Index of an output with an empty script.
    EmptyScriptOutputIndex, "empty_script_output_index", ["emptyout", "empty_script_output_index"], 0
);
typed_index!(
    /// Index of an output with an unrecognized script.
    UnknownScriptOutputIndex, "unknown_script_output_index", ["unknownout", "unknown_script_output_index"], 200_000
);
