#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::OutputType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

impl SpendableTypeId {
    /// `None` for OP_RETURN, the one unspendable type.
    #[inline]
    pub const fn from_output_type(value: OutputType) -> Option<Self> {
        match value {
            OutputType::P2PK65 | OutputType::P2PK33 => Some(Self::P2PK),
            OutputType::P2PKH => Some(Self::P2PKH),
            OutputType::P2MS => Some(Self::P2MS),
            OutputType::P2SH => Some(Self::P2SH),
            OutputType::P2WPKH => Some(Self::P2WPKH),
            OutputType::P2WSH => Some(Self::P2WSH),
            OutputType::P2TR => Some(Self::P2TR),
            OutputType::P2A => Some(Self::P2A),
            OutputType::Empty => Some(Self::EmptyScript),
            OutputType::Unknown => Some(Self::UnknownScript),
            OutputType::OpReturn => None,
        }
    }

    /// The type's member key, the word its series ids build on (`p2pk_output_count`,
    /// `empty_script_output_count`, `empty_script_supply`).
    pub const fn key(self) -> &'static str {
        match self {
            Self::P2PK => "p2pk",
            Self::P2PKH => "p2pkh",
            Self::P2MS => "p2ms",
            Self::P2SH => "p2sh",
            Self::P2WPKH => "p2wpkh",
            Self::P2WSH => "p2wsh",
            Self::P2TR => "p2tr",
            Self::P2A => "p2a",
            Self::EmptyScript => "empty_script",
            Self::UnknownScript => "unknown_script",
        }
    }

    pub const fn cohort(self) -> CohortId {
        CohortId::Type(self)
    }
}

pub const SPENDABLE_TYPE_NAMES: SpendableType<CohortName> = SpendableType {
    p2pk: CohortName::new("p2pk", "P2PK", "Pay to Public Key"),
    p2pkh: CohortName::new("p2pkh", "P2PKH", "Pay to Public Key Hash"),
    p2ms: CohortName::new("p2ms", "P2MS", "Pay to Multisig"),
    p2sh: CohortName::new("p2sh", "P2SH", "Pay to Script Hash"),
    p2wpkh: CohortName::new("p2wpkh", "P2WPKH", "Pay to Witness Public Key Hash"),
    p2wsh: CohortName::new("p2wsh", "P2WSH", "Pay to Witness Script Hash"),
    p2tr: CohortName::new("p2tr", "P2TR", "Pay to Taproot"),
    p2a: CohortName::new("p2a", "P2A", "Pay to Anchor"),
    empty_script: CohortName::new("empty_script", "Empty", "Empty Script"),
    unknown_script: CohortName::new("unknown_script", "Unknown", "Unknown Script"),
};

#[derive(Default, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct SpendableType<T> {
    /// Uses pay-to-public-key outputs, with a 33- or 65-byte key.
    p2pk: T,
    /// Uses pay-to-public-key-hash outputs.
    p2pkh: T,
    /// Uses bare pay-to-multisig outputs.
    p2ms: T,
    /// Uses pay-to-script-hash outputs.
    p2sh: T,
    /// Uses version-0 pay-to-witness-public-key-hash outputs.
    p2wpkh: T,
    /// Uses version-0 pay-to-witness-script-hash outputs.
    p2wsh: T,
    /// Uses pay-to-Taproot outputs.
    p2tr: T,
    /// Uses pay-to-Anchor outputs.
    p2a: T,
    /// Uses outputs with an empty locking script.
    empty_script: T,
    /// Uses outputs that do not match another recognized locking-script type.
    unknown_script: T,
}

// Spendable output types as analytics count them: P2PK is one type whatever its key's size
// (the indexer stores 33- and 65-byte keys apart).
define_cohort_id!(SpendableTypeId for SpendableType {
    P2PK => p2pk,
    P2PKH => p2pkh,
    P2MS => p2ms,
    P2SH => p2sh,
    P2WPKH => p2wpkh,
    P2WSH => p2wsh,
    P2TR => p2tr,
    P2A => p2a,
    EmptyScript => empty_script,
    UnknownScript => unknown_script,
});

impl<T> SpendableType<T> {
    /// The member holding `output_type`; P2PK's 33- and 65-byte keys share one.
    pub fn get(&self, output_type: OutputType) -> &T {
        SpendableTypeId::from_output_type(output_type)
            .expect("spendable output type")
            .select(self)
    }

    pub fn get_mut(&mut self, output_type: OutputType) -> &mut T {
        SpendableTypeId::from_output_type(output_type)
            .expect("spendable output type")
            .select_mut(self)
    }

    pub fn iter_typed_mut(&mut self) -> impl Iterator<Item = (SpendableTypeId, &mut T)> {
        SpendableTypeId::ALL.iter().copied().zip(self.iter_mut())
    }
}
