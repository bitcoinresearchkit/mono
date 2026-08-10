use std::ops::{Add, AddAssign};

use brk_traversable::Traversable;
use brk_types::OutputType;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;
use vecdb::{ColumnId, VecValue, Version};

use super::{CohortName, Filter};

pub const SPENDABLE_TYPE_COUNT: usize = 11;

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

pub const SPENDABLE_TYPE_IDS: [SpendableTypeId; SPENDABLE_TYPE_COUNT] = [
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
    pub const fn from_output_type(value: OutputType) -> Option<Self> {
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

impl ColumnId for SpendableTypeId {
    type Row<T>
        = [T; SPENDABLE_TYPE_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &SPENDABLE_TYPE_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self as usize]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self as usize]
    }

    #[inline]
    fn from_fn<T, F>(mut f: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        std::array::from_fn(|index| f(SPENDABLE_TYPE_IDS[index]))
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, f: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(f)
    }
}

/// Spendable type values
pub const SPENDABLE_TYPE_VALUES: SpendableType<OutputType> = SpendableType {
    p2pk65: OutputType::P2PK65,
    p2pk33: OutputType::P2PK33,
    p2pkh: OutputType::P2PKH,
    p2ms: OutputType::P2MS,
    p2sh: OutputType::P2SH,
    p2wpkh: OutputType::P2WPKH,
    p2wsh: OutputType::P2WSH,
    p2tr: OutputType::P2TR,
    p2a: OutputType::P2A,
    unknown: OutputType::Unknown,
    empty: OutputType::Empty,
};

/// Spendable type filters
pub const SPENDABLE_TYPE_FILTERS: SpendableType<Filter> = SpendableType {
    p2pk65: Filter::Type(SPENDABLE_TYPE_VALUES.p2pk65),
    p2pk33: Filter::Type(SPENDABLE_TYPE_VALUES.p2pk33),
    p2pkh: Filter::Type(SPENDABLE_TYPE_VALUES.p2pkh),
    p2ms: Filter::Type(SPENDABLE_TYPE_VALUES.p2ms),
    p2sh: Filter::Type(SPENDABLE_TYPE_VALUES.p2sh),
    p2wpkh: Filter::Type(SPENDABLE_TYPE_VALUES.p2wpkh),
    p2wsh: Filter::Type(SPENDABLE_TYPE_VALUES.p2wsh),
    p2tr: Filter::Type(SPENDABLE_TYPE_VALUES.p2tr),
    p2a: Filter::Type(SPENDABLE_TYPE_VALUES.p2a),
    unknown: Filter::Type(SPENDABLE_TYPE_VALUES.unknown),
    empty: Filter::Type(SPENDABLE_TYPE_VALUES.empty),
};

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

#[derive(Default, Clone, Debug, Traversable, Serialize)]
pub struct SpendableType<T> {
    pub p2pk65: T,
    pub p2pk33: T,
    pub p2pkh: T,
    pub p2ms: T,
    pub p2sh: T,
    pub p2wpkh: T,
    pub p2wsh: T,
    pub p2tr: T,
    pub p2a: T,
    pub unknown: T,
    pub empty: T,
}

impl SpendableType<CohortName> {
    pub const fn names() -> &'static Self {
        &SPENDABLE_TYPE_NAMES
    }
}

impl<T> SpendableType<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        let f = SPENDABLE_TYPE_FILTERS;
        let n = SPENDABLE_TYPE_NAMES;
        Self {
            p2pk65: create(f.p2pk65, n.p2pk65.id),
            p2pk33: create(f.p2pk33, n.p2pk33.id),
            p2pkh: create(f.p2pkh, n.p2pkh.id),
            p2ms: create(f.p2ms, n.p2ms.id),
            p2sh: create(f.p2sh, n.p2sh.id),
            p2wpkh: create(f.p2wpkh, n.p2wpkh.id),
            p2wsh: create(f.p2wsh, n.p2wsh.id),
            p2tr: create(f.p2tr, n.p2tr.id),
            p2a: create(f.p2a, n.p2a.id),
            unknown: create(f.unknown, n.unknown.id),
            empty: create(f.empty, n.empty.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        let f = SPENDABLE_TYPE_FILTERS;
        let n = SPENDABLE_TYPE_NAMES;
        Ok(Self {
            p2pk65: create(f.p2pk65, n.p2pk65.id)?,
            p2pk33: create(f.p2pk33, n.p2pk33.id)?,
            p2pkh: create(f.p2pkh, n.p2pkh.id)?,
            p2ms: create(f.p2ms, n.p2ms.id)?,
            p2sh: create(f.p2sh, n.p2sh.id)?,
            p2wpkh: create(f.p2wpkh, n.p2wpkh.id)?,
            p2wsh: create(f.p2wsh, n.p2wsh.id)?,
            p2tr: create(f.p2tr, n.p2tr.id)?,
            p2a: create(f.p2a, n.p2a.id)?,
            unknown: create(f.unknown, n.unknown.id)?,
            empty: create(f.empty, n.empty.id)?,
        })
    }

    pub fn get(&self, output_type: OutputType) -> &T {
        match output_type {
            OutputType::P2PK65 => &self.p2pk65,
            OutputType::P2PK33 => &self.p2pk33,
            OutputType::P2PKH => &self.p2pkh,
            OutputType::P2MS => &self.p2ms,
            OutputType::P2SH => &self.p2sh,
            OutputType::P2WPKH => &self.p2wpkh,
            OutputType::P2WSH => &self.p2wsh,
            OutputType::P2TR => &self.p2tr,
            OutputType::P2A => &self.p2a,
            OutputType::Unknown => &self.unknown,
            OutputType::Empty => &self.empty,
            _ => unreachable!(),
        }
    }

    pub fn get_mut(&mut self, output_type: OutputType) -> &mut T {
        match output_type {
            OutputType::P2PK65 => &mut self.p2pk65,
            OutputType::P2PK33 => &mut self.p2pk33,
            OutputType::P2PKH => &mut self.p2pkh,
            OutputType::P2MS => &mut self.p2ms,
            OutputType::P2SH => &mut self.p2sh,
            OutputType::P2WPKH => &mut self.p2wpkh,
            OutputType::P2WSH => &mut self.p2wsh,
            OutputType::P2TR => &mut self.p2tr,
            OutputType::P2A => &mut self.p2a,
            OutputType::Unknown => &mut self.unknown,
            OutputType::Empty => &mut self.empty,
            _ => unreachable!(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self.p2pk65,
            &self.p2pk33,
            &self.p2pkh,
            &self.p2ms,
            &self.p2sh,
            &self.p2wpkh,
            &self.p2wsh,
            &self.p2tr,
            &self.p2a,
            &self.unknown,
            &self.empty,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.p2pk65,
            &mut self.p2pk33,
            &mut self.p2pkh,
            &mut self.p2ms,
            &mut self.p2sh,
            &mut self.p2wpkh,
            &mut self.p2wsh,
            &mut self.p2tr,
            &mut self.p2a,
            &mut self.unknown,
            &mut self.empty,
        ]
        .into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self.p2pk65,
            &mut self.p2pk33,
            &mut self.p2pkh,
            &mut self.p2ms,
            &mut self.p2sh,
            &mut self.p2wpkh,
            &mut self.p2wsh,
            &mut self.p2tr,
            &mut self.p2a,
            &mut self.unknown,
            &mut self.empty,
        ]
        .into_par_iter()
    }

    pub fn iter_typed(&self) -> impl Iterator<Item = (OutputType, &T)> {
        [
            (OutputType::P2PK65, &self.p2pk65),
            (OutputType::P2PK33, &self.p2pk33),
            (OutputType::P2PKH, &self.p2pkh),
            (OutputType::P2MS, &self.p2ms),
            (OutputType::P2SH, &self.p2sh),
            (OutputType::P2WPKH, &self.p2wpkh),
            (OutputType::P2WSH, &self.p2wsh),
            (OutputType::P2TR, &self.p2tr),
            (OutputType::P2A, &self.p2a),
            (OutputType::Unknown, &self.unknown),
            (OutputType::Empty, &self.empty),
        ]
        .into_iter()
    }

    pub fn iter_typed_mut(&mut self) -> impl Iterator<Item = (OutputType, &mut T)> {
        [
            (OutputType::P2PK65, &mut self.p2pk65),
            (OutputType::P2PK33, &mut self.p2pk33),
            (OutputType::P2PKH, &mut self.p2pkh),
            (OutputType::P2MS, &mut self.p2ms),
            (OutputType::P2SH, &mut self.p2sh),
            (OutputType::P2WPKH, &mut self.p2wpkh),
            (OutputType::P2WSH, &mut self.p2wsh),
            (OutputType::P2TR, &mut self.p2tr),
            (OutputType::P2A, &mut self.p2a),
            (OutputType::Unknown, &mut self.unknown),
            (OutputType::Empty, &mut self.empty),
        ]
        .into_iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_ids_match_spendable_type_order() {
        let output_types: Vec<_> = SPENDABLE_TYPE_VALUES.iter().copied().collect();
        let column_output_types: Vec<_> = SpendableTypeId::ALL
            .iter()
            .map(|column| column.output_type())
            .collect();

        assert_eq!(column_output_types, output_types);
        assert_eq!(
            SpendableTypeId::from_output_type(OutputType::OpReturn),
            None
        );

        let row = SpendableTypeId::from_fn(|column| column.index());
        for column in SpendableTypeId::ALL {
            assert_eq!(*column.get(&row), column.index());
        }
    }
}
