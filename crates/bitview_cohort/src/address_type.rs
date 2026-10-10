#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::OutputType;

use super::SpendableTypeId;

/// Address types as analytics count them: P2PK is one type whatever its key's size, and P2A,
/// one fixed anyone-can-spend script, is not an address type. The indexer and the address
/// lookups keep their own per-script split.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct AddressType<T> {
    /// Uses pay-to-public-key addresses, with a 33- or 65-byte key.
    p2pk: T,
    /// Uses pay-to-public-key-hash addresses.
    p2pkh: T,
    /// Uses pay-to-script-hash addresses.
    p2sh: T,
    /// Uses version-0 pay-to-witness-public-key-hash addresses.
    p2wpkh: T,
    /// Uses version-0 pay-to-witness-script-hash addresses.
    p2wsh: T,
    /// Uses pay-to-Taproot addresses.
    p2tr: T,
}

define_cohort_id!(
    AddressTypeId for AddressType {
        P2PK => p2pk,
        P2PKH => p2pkh,
        P2SH => p2sh,
        P2WPKH => p2wpkh,
        P2WSH => p2wsh,
        P2TR => p2tr,
    }
);

impl AddressTypeId {
    /// `None` for outputs without an address type, P2A included.
    #[inline]
    pub const fn from_output_type(output_type: OutputType) -> Option<Self> {
        match output_type {
            OutputType::P2PK65 | OutputType::P2PK33 => Some(Self::P2PK),
            OutputType::P2PKH => Some(Self::P2PKH),
            OutputType::P2SH => Some(Self::P2SH),
            OutputType::P2WPKH => Some(Self::P2WPKH),
            OutputType::P2WSH => Some(Self::P2WSH),
            OutputType::P2TR => Some(Self::P2TR),
            _ => None,
        }
    }

    /// The spendable output type the address type's outputs have.
    pub const fn spendable(self) -> SpendableTypeId {
        match self {
            Self::P2PK => SpendableTypeId::P2PK,
            Self::P2PKH => SpendableTypeId::P2PKH,
            Self::P2SH => SpendableTypeId::P2SH,
            Self::P2WPKH => SpendableTypeId::P2WPKH,
            Self::P2WSH => SpendableTypeId::P2WSH,
            Self::P2TR => SpendableTypeId::P2TR,
        }
    }

    /// The member key, the word its series ids start with (`p2pk_address_count`).
    pub const fn key(self) -> &'static str {
        self.spendable().key()
    }
}

impl<T> AddressType<T> {
    /// The member holding `output_type`, if it has an address type.
    pub fn get_mut(&mut self, output_type: OutputType) -> Option<&mut T> {
        AddressTypeId::from_output_type(output_type).map(|id| id.select_mut(self))
    }

    pub fn iter_typed(&self) -> impl Iterator<Item = (AddressTypeId, &T)> {
        AddressTypeId::ALL.iter().copied().zip(self.iter())
    }

    pub fn iter_typed_mut(&mut self) -> impl Iterator<Item = (AddressTypeId, &mut T)> {
        AddressTypeId::ALL.iter().copied().zip(self.iter_mut())
    }
}
