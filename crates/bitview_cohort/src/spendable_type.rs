use std::ops::{Add, AddAssign};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::OutputType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SpendableTypeId {
    P2PK65,
    P2PK33,
    P2PKH,
    P2MS,
    P2SH,
    P2WPKH,
    P2WSH,
    P2TR,
    P2A,
    Unknown,
    Empty,
}

const SPENDABLE_TYPE_COUNT: usize = 11;

const SPENDABLE_TYPE_IDS: [SpendableTypeId; SPENDABLE_TYPE_COUNT] = [
    SpendableTypeId::P2PK65,
    SpendableTypeId::P2PK33,
    SpendableTypeId::P2PKH,
    SpendableTypeId::P2MS,
    SpendableTypeId::P2SH,
    SpendableTypeId::P2WPKH,
    SpendableTypeId::P2WSH,
    SpendableTypeId::P2TR,
    SpendableTypeId::P2A,
    SpendableTypeId::Unknown,
    SpendableTypeId::Empty,
];

impl SpendableTypeId {
    #[inline]
    pub fn select<T>(self, values: &SpendableType<T>) -> &T {
        match self {
            Self::P2PK65 => &values.p2pk65,
            Self::P2PK33 => &values.p2pk33,
            Self::P2PKH => &values.p2pkh,
            Self::P2MS => &values.p2ms,
            Self::P2SH => &values.p2sh,
            Self::P2WPKH => &values.p2wpkh,
            Self::P2WSH => &values.p2wsh,
            Self::P2TR => &values.p2tr,
            Self::P2A => &values.p2a,
            Self::Unknown => &values.unknown,
            Self::Empty => &values.empty,
        }
    }

    #[inline]
    fn select_mut<T>(self, values: &mut SpendableType<T>) -> &mut T {
        match self {
            Self::P2PK65 => &mut values.p2pk65,
            Self::P2PK33 => &mut values.p2pk33,
            Self::P2PKH => &mut values.p2pkh,
            Self::P2MS => &mut values.p2ms,
            Self::P2SH => &mut values.p2sh,
            Self::P2WPKH => &mut values.p2wpkh,
            Self::P2WSH => &mut values.p2wsh,
            Self::P2TR => &mut values.p2tr,
            Self::P2A => &mut values.p2a,
            Self::Unknown => &mut values.unknown,
            Self::Empty => &mut values.empty,
        }
    }

    pub(crate) const fn from_output_type(value: OutputType) -> Option<Self> {
        match value {
            OutputType::P2PK65 => Some(Self::P2PK65),
            OutputType::P2PK33 => Some(Self::P2PK33),
            OutputType::P2PKH => Some(Self::P2PKH),
            OutputType::P2MS => Some(Self::P2MS),
            OutputType::P2SH => Some(Self::P2SH),
            OutputType::P2WPKH => Some(Self::P2WPKH),
            OutputType::P2WSH => Some(Self::P2WSH),
            OutputType::P2TR => Some(Self::P2TR),
            OutputType::P2A => Some(Self::P2A),
            OutputType::Unknown => Some(Self::Unknown),
            OutputType::Empty => Some(Self::Empty),
            OutputType::OpReturn => None,
        }
    }

    pub const fn output_type(self) -> OutputType {
        match self {
            Self::P2PK65 => OutputType::P2PK65,
            Self::P2PK33 => OutputType::P2PK33,
            Self::P2PKH => OutputType::P2PKH,
            Self::P2MS => OutputType::P2MS,
            Self::P2SH => OutputType::P2SH,
            Self::P2WPKH => OutputType::P2WPKH,
            Self::P2WSH => OutputType::P2WSH,
            Self::P2TR => OutputType::P2TR,
            Self::P2A => OutputType::P2A,
            Self::Unknown => OutputType::Unknown,
            Self::Empty => OutputType::Empty,
        }
    }
}

impl SpendableTypeId {
    pub const ALL: &'static [Self] = &SPENDABLE_TYPE_IDS;
}

/// Spendable type names
pub const SPENDABLE_TYPE_NAMES: SpendableType<CohortName> = SpendableType {
    p2pk65: CohortName::new("p2pk65", "P2PK65", "Pay to Public Key (65 bytes)"),
    p2pk33: CohortName::new("p2pk33", "P2PK33", "Pay to Public Key (33 bytes)"),
    p2pkh: CohortName::new("p2pkh", "P2PKH", "Pay to Public Key Hash"),
    p2ms: CohortName::new("p2ms", "P2MS", "Pay to Multisig"),
    p2sh: CohortName::new("p2sh", "P2SH", "Pay to Script Hash"),
    p2wpkh: CohortName::new("p2wpkh", "P2WPKH", "Pay to Witness Public Key Hash"),
    p2wsh: CohortName::new("p2wsh", "P2WSH", "Pay to Witness Script Hash"),
    p2tr: CohortName::new("p2tr", "P2TR", "Pay to Taproot"),
    p2a: CohortName::new("p2a", "P2A", "Pay to Anchor"),
    unknown: CohortName::new("unknown_outputs", "Unknown", "Unknown Output Type"),
    empty: CohortName::new("empty_outputs", "Empty", "Empty Output"),
};

#[derive(Default, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct SpendableType<T> {
    /// Uses pay-to-public-key outputs with a 65-byte key field.
    p2pk65: T,
    /// Uses pay-to-public-key outputs with a 33-byte key field.
    p2pk33: T,
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
    /// Uses outputs that do not match another recognized locking-script type.
    unknown: T,
    /// Uses outputs with an empty locking script.
    empty: T,
}

impl_cohort_collection!(SpendableTypeId for SpendableType {
    P2PK65 => p2pk65,
    P2PK33 => p2pk33,
    P2PKH => p2pkh,
    P2MS => p2ms,
    P2SH => p2sh,
    P2WPKH => p2wpkh,
    P2WSH => p2wsh,
    P2TR => p2tr,
    P2A => p2a,
    Unknown => unknown,
    Empty => empty,
});

impl<T> SpendableType<T> {
    pub fn new(mut create: impl FnMut(CohortId) -> T) -> Self {
        Self::from_fn(|kind| create(CohortId::Type(kind.output_type())))
    }

    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Self::try_from_fn(|kind| create(CohortId::Type(kind.output_type())))
    }

    pub fn map_with_id<U>(&self, mut map: impl FnMut(CohortId, &T) -> U) -> SpendableType<U> {
        SpendableType::from_fn(|kind| map(CohortId::Type(kind.output_type()), kind.select(self)))
    }

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

    pub fn iter_typed(&self) -> impl Iterator<Item = (OutputType, &T)> {
        SpendableTypeId::ALL
            .iter()
            .map(|id| id.output_type())
            .zip(self.iter())
    }

    pub fn iter_typed_mut(&mut self) -> impl Iterator<Item = (OutputType, &mut T)> {
        SpendableTypeId::ALL
            .iter()
            .map(|id| id.output_type())
            .zip(self.iter_mut())
    }
}

impl<T> Add for SpendableType<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            p2pk65: self.p2pk65 + rhs.p2pk65,
            p2pk33: self.p2pk33 + rhs.p2pk33,
            p2pkh: self.p2pkh + rhs.p2pkh,
            p2ms: self.p2ms + rhs.p2ms,
            p2sh: self.p2sh + rhs.p2sh,
            p2wpkh: self.p2wpkh + rhs.p2wpkh,
            p2wsh: self.p2wsh + rhs.p2wsh,
            p2tr: self.p2tr + rhs.p2tr,
            p2a: self.p2a + rhs.p2a,
            unknown: self.unknown + rhs.unknown,
            empty: self.empty + rhs.empty,
        }
    }
}

impl<T> AddAssign for SpendableType<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.p2pk65 += rhs.p2pk65;
        self.p2pk33 += rhs.p2pk33;
        self.p2pkh += rhs.p2pkh;
        self.p2ms += rhs.p2ms;
        self.p2sh += rhs.p2sh;
        self.p2wpkh += rhs.p2wpkh;
        self.p2wsh += rhs.p2wsh;
        self.p2tr += rhs.p2tr;
        self.p2a += rhs.p2a;
        self.unknown += rhs.unknown;
        self.empty += rhs.empty;
    }
}
