use std::ops::{Add, AddAssign};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::OutputType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

/// Spendable output types as analytics count them: P2PK is one type whatever its key's size
/// (the indexer stores 33- and 65-byte keys apart).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SpendableTypeId {
    P2PK,
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

impl SpendableTypeId {
    pub const ALL: &'static [Self] = &[
        Self::P2PK,
        Self::P2PKH,
        Self::P2MS,
        Self::P2SH,
        Self::P2WPKH,
        Self::P2WSH,
        Self::P2TR,
        Self::P2A,
        Self::Unknown,
        Self::Empty,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn select<T>(self, values: &SpendableType<T>) -> &T {
        match self {
            Self::P2PK => &values.p2pk,
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
    pub fn select_mut<T>(self, values: &mut SpendableType<T>) -> &mut T {
        match self {
            Self::P2PK => &mut values.p2pk,
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
            OutputType::Unknown => Some(Self::Unknown),
            OutputType::Empty => Some(Self::Empty),
            OutputType::OpReturn => None,
        }
    }

    /// The type's member key, the word per-entry series ids build on (`p2pk_output_count`,
    /// `empty_output_count`); type cohort names spell out `empty_output` and `unknown_output`
    /// (`empty_output_supply`).
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
            Self::Unknown => "unknown",
            Self::Empty => "empty",
        }
    }

    pub const fn cohort(self) -> CohortId {
        CohortId::Type(self)
    }
}

/// Spendable type names
pub const SPENDABLE_TYPE_NAMES: SpendableType<CohortName> = SpendableType {
    p2pk: CohortName::new("p2pk", "P2PK", "Pay to Public Key"),
    p2pkh: CohortName::new("p2pkh", "P2PKH", "Pay to Public Key Hash"),
    p2ms: CohortName::new("p2ms", "P2MS", "Pay to Multisig"),
    p2sh: CohortName::new("p2sh", "P2SH", "Pay to Script Hash"),
    p2wpkh: CohortName::new("p2wpkh", "P2WPKH", "Pay to Witness Public Key Hash"),
    p2wsh: CohortName::new("p2wsh", "P2WSH", "Pay to Witness Script Hash"),
    p2tr: CohortName::new("p2tr", "P2TR", "Pay to Taproot"),
    p2a: CohortName::new("p2a", "P2A", "Pay to Anchor"),
    unknown: CohortName::new("unknown_output", "Unknown", "Unknown Output Type"),
    empty: CohortName::new("empty_output", "Empty", "Empty Output"),
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
    /// Uses outputs that do not match another recognized locking-script type.
    unknown: T,
    /// Uses outputs with an empty locking script.
    empty: T,
}

impl_cohort_collection!(SpendableTypeId for SpendableType {
    P2PK => p2pk,
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
        Self::from_fn(|kind| create(kind.cohort()))
    }

    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Self::try_from_fn(|kind| create(kind.cohort()))
    }

    pub fn map_with_id<U>(&self, mut map: impl FnMut(CohortId, &T) -> U) -> SpendableType<U> {
        SpendableType::from_fn(|kind| map(kind.cohort(), kind.select(self)))
    }

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

    pub fn iter_typed(&self) -> impl Iterator<Item = (SpendableTypeId, &T)> {
        SpendableTypeId::ALL.iter().copied().zip(self.iter())
    }

    pub fn iter_typed_mut(&mut self) -> impl Iterator<Item = (SpendableTypeId, &mut T)> {
        SpendableTypeId::ALL.iter().copied().zip(self.iter_mut())
    }
}

impl<T> Add for SpendableType<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            p2pk: self.p2pk + rhs.p2pk,
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
        self.p2pk += rhs.p2pk;
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
