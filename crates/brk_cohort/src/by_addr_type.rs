use std::ops::{Add, AddAssign};

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::OutputType;
use rayon::prelude::*;
use vecdb::{ColumnId, VecValue, Version};

use super::Filter;

pub const P2PK65: &str = "p2pk65";
pub const P2PK33: &str = "p2pk33";
pub const P2PKH: &str = "p2pkh";
pub const P2SH: &str = "p2sh";
pub const P2WPKH: &str = "p2wpkh";
pub const P2WSH: &str = "p2wsh";
pub const P2TR: &str = "p2tr";
pub const P2A: &str = "p2a";

pub const ADDR_TYPE_COUNT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum AddrTypeId {
    P2PK65,
    P2PK33,
    P2PKH,
    P2SH,
    P2WPKH,
    P2WSH,
    P2TR,
    P2A,
}

pub const ADDR_TYPE_IDS: [AddrTypeId; ADDR_TYPE_COUNT] = [
    AddrTypeId::P2PK65,
    AddrTypeId::P2PK33,
    AddrTypeId::P2PKH,
    AddrTypeId::P2SH,
    AddrTypeId::P2WPKH,
    AddrTypeId::P2WSH,
    AddrTypeId::P2TR,
    AddrTypeId::P2A,
];

impl ColumnId for AddrTypeId {
    type Row<T>
        = [T; ADDR_TYPE_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &ADDR_TYPE_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self.index()]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self.index()]
    }

    #[inline]
    fn from_fn<T, F>(mut create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        std::array::from_fn(|index| create(ADDR_TYPE_IDS[index]))
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, create: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(create)
    }
}

impl AddrTypeId {
    pub const fn name(self) -> &'static str {
        match self {
            Self::P2PK65 => P2PK65,
            Self::P2PK33 => P2PK33,
            Self::P2PKH => P2PKH,
            Self::P2SH => P2SH,
            Self::P2WPKH => P2WPKH,
            Self::P2WSH => P2WSH,
            Self::P2TR => P2TR,
            Self::P2A => P2A,
        }
    }

    pub const fn output_type(self) -> OutputType {
        match self {
            Self::P2PK65 => OutputType::P2PK65,
            Self::P2PK33 => OutputType::P2PK33,
            Self::P2PKH => OutputType::P2PKH,
            Self::P2SH => OutputType::P2SH,
            Self::P2WPKH => OutputType::P2WPKH,
            Self::P2WSH => OutputType::P2WSH,
            Self::P2TR => OutputType::P2TR,
            Self::P2A => OutputType::P2A,
        }
    }

    pub fn select<T>(self, values: &ByAddrType<T>) -> &T {
        values.get_unwrap(self.output_type())
    }

    pub fn select_mut<T>(self, values: &mut ByAddrType<T>) -> &mut T {
        values.get_mut_unwrap(self.output_type())
    }

    pub fn series<T>(mut create: impl FnMut(Self, &'static str) -> T) -> ByAddrType<T> {
        ByAddrType::from_fn(|id| create(id, id.name()))
    }
}

#[derive(Default, Clone, Debug, Traversable)]
pub struct ByAddrType<T> {
    pub p2pk65: T,
    pub p2pk33: T,
    pub p2pkh: T,
    pub p2sh: T,
    pub p2wpkh: T,
    pub p2wsh: T,
    pub p2tr: T,
    pub p2a: T,
}

impl<T> ByAddrType<T> {
    pub fn from_fn(mut create: impl FnMut(AddrTypeId) -> T) -> Self {
        Self {
            p2pk65: create(AddrTypeId::P2PK65),
            p2pk33: create(AddrTypeId::P2PK33),
            p2pkh: create(AddrTypeId::P2PKH),
            p2sh: create(AddrTypeId::P2SH),
            p2wpkh: create(AddrTypeId::P2WPKH),
            p2wsh: create(AddrTypeId::P2WSH),
            p2tr: create(AddrTypeId::P2TR),
            p2a: create(AddrTypeId::P2A),
        }
    }

    pub fn try_from_fn<E>(mut create: impl FnMut(AddrTypeId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            p2pk65: create(AddrTypeId::P2PK65)?,
            p2pk33: create(AddrTypeId::P2PK33)?,
            p2pkh: create(AddrTypeId::P2PKH)?,
            p2sh: create(AddrTypeId::P2SH)?,
            p2wpkh: create(AddrTypeId::P2WPKH)?,
            p2wsh: create(AddrTypeId::P2WSH)?,
            p2tr: create(AddrTypeId::P2TR)?,
            p2a: create(AddrTypeId::P2A)?,
        })
    }

    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter) -> T,
    {
        Self::from_fn(|id| create(Filter::Type(id.output_type())))
    }

    pub fn new_with_name<F>(f: F) -> Result<Self>
    where
        F: Fn(&'static str) -> Result<T>,
    {
        Self::try_from_fn(|id| f(id.name()))
    }

    pub fn map_with_name<U>(&self, f: impl Fn(&'static str, &T) -> U) -> ByAddrType<U> {
        ByAddrType {
            p2pk65: f(P2PK65, &self.p2pk65),
            p2pk33: f(P2PK33, &self.p2pk33),
            p2pkh: f(P2PKH, &self.p2pkh),
            p2sh: f(P2SH, &self.p2sh),
            p2wpkh: f(P2WPKH, &self.p2wpkh),
            p2wsh: f(P2WSH, &self.p2wsh),
            p2tr: f(P2TR, &self.p2tr),
            p2a: f(P2A, &self.p2a),
        }
    }

    pub fn new_with_index<F>(f: F) -> Result<Self>
    where
        F: Fn(usize) -> Result<T>,
    {
        Self::try_from_fn(|id| f(id as usize))
    }

    #[inline]
    pub fn get_unwrap(&self, addr_type: OutputType) -> &T {
        self.get(addr_type).unwrap()
    }

    #[inline]
    pub fn get(&self, addr_type: OutputType) -> Option<&T> {
        match addr_type {
            OutputType::P2PK65 => Some(&self.p2pk65),
            OutputType::P2PK33 => Some(&self.p2pk33),
            OutputType::P2PKH => Some(&self.p2pkh),
            OutputType::P2SH => Some(&self.p2sh),
            OutputType::P2WPKH => Some(&self.p2wpkh),
            OutputType::P2WSH => Some(&self.p2wsh),
            OutputType::P2TR => Some(&self.p2tr),
            OutputType::P2A => Some(&self.p2a),
            _ => None,
        }
    }

    #[inline]
    pub fn get_mut_unwrap(&mut self, addr_type: OutputType) -> &mut T {
        self.get_mut(addr_type).unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self, addr_type: OutputType) -> Option<&mut T> {
        match addr_type {
            OutputType::P2PK65 => Some(&mut self.p2pk65),
            OutputType::P2PK33 => Some(&mut self.p2pk33),
            OutputType::P2PKH => Some(&mut self.p2pkh),
            OutputType::P2SH => Some(&mut self.p2sh),
            OutputType::P2WPKH => Some(&mut self.p2wpkh),
            OutputType::P2WSH => Some(&mut self.p2wsh),
            OutputType::P2TR => Some(&mut self.p2tr),
            OutputType::P2A => Some(&mut self.p2a),
            _ => None,
        }
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &T> {
        [
            &self.p2pk65,
            &self.p2pk33,
            &self.p2pkh,
            &self.p2sh,
            &self.p2wpkh,
            &self.p2wsh,
            &self.p2tr,
            &self.p2a,
        ]
        .into_iter()
    }

    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.p2pk65,
            &mut self.p2pk33,
            &mut self.p2pkh,
            &mut self.p2sh,
            &mut self.p2wpkh,
            &mut self.p2wsh,
            &mut self.p2tr,
            &mut self.p2a,
        ]
        .into_iter()
    }

    #[inline]
    pub fn par_values(&mut self) -> impl ParallelIterator<Item = &T>
    where
        T: Send + Sync,
    {
        [
            &self.p2pk65,
            &self.p2pk33,
            &self.p2pkh,
            &self.p2sh,
            &self.p2wpkh,
            &self.p2wsh,
            &self.p2tr,
            &self.p2a,
        ]
        .into_par_iter()
    }

    #[inline]
    pub fn par_values_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [
            &mut self.p2pk65,
            &mut self.p2pk33,
            &mut self.p2pkh,
            &mut self.p2sh,
            &mut self.p2wpkh,
            &mut self.p2wsh,
            &mut self.p2tr,
            &mut self.p2a,
        ]
        .into_par_iter()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (OutputType, &T)> {
        [
            (OutputType::P2PK65, &self.p2pk65),
            (OutputType::P2PK33, &self.p2pk33),
            (OutputType::P2PKH, &self.p2pkh),
            (OutputType::P2SH, &self.p2sh),
            (OutputType::P2WPKH, &self.p2wpkh),
            (OutputType::P2WSH, &self.p2wsh),
            (OutputType::P2TR, &self.p2tr),
            (OutputType::P2A, &self.p2a),
        ]
        .into_iter()
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = (OutputType, T)> {
        [
            (OutputType::P2PK65, self.p2pk65),
            (OutputType::P2PK33, self.p2pk33),
            (OutputType::P2PKH, self.p2pkh),
            (OutputType::P2SH, self.p2sh),
            (OutputType::P2WPKH, self.p2wpkh),
            (OutputType::P2WSH, self.p2wsh),
            (OutputType::P2TR, self.p2tr),
            (OutputType::P2A, self.p2a),
        ]
        .into_iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (OutputType, &mut T)> {
        [
            (OutputType::P2PK65, &mut self.p2pk65),
            (OutputType::P2PK33, &mut self.p2pk33),
            (OutputType::P2PKH, &mut self.p2pkh),
            (OutputType::P2SH, &mut self.p2sh),
            (OutputType::P2WPKH, &mut self.p2wpkh),
            (OutputType::P2WSH, &mut self.p2wsh),
            (OutputType::P2TR, &mut self.p2tr),
            (OutputType::P2A, &mut self.p2a),
        ]
        .into_iter()
    }
}

impl<T> Add for ByAddrType<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            p2pk65: self.p2pk65 + rhs.p2pk65,
            p2pk33: self.p2pk33 + rhs.p2pk33,
            p2pkh: self.p2pkh + rhs.p2pkh,
            p2sh: self.p2sh + rhs.p2sh,
            p2wpkh: self.p2wpkh + rhs.p2wpkh,
            p2wsh: self.p2wsh + rhs.p2wsh,
            p2tr: self.p2tr + rhs.p2tr,
            p2a: self.p2a + rhs.p2a,
        }
    }
}

impl<T> AddAssign for ByAddrType<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.p2pk65 += rhs.p2pk65;
        self.p2pk33 += rhs.p2pk33;
        self.p2pkh += rhs.p2pkh;
        self.p2sh += rhs.p2sh;
        self.p2wpkh += rhs.p2wpkh;
        self.p2wsh += rhs.p2wsh;
        self.p2tr += rhs.p2tr;
        self.p2a += rhs.p2a;
    }
}

impl<T> ByAddrType<Option<T>> {
    pub fn take(&mut self) {
        self.values_mut().for_each(|opt| {
            opt.take();
        });
    }
}

#[cfg(test)]
mod tests {
    use vecdb::ColumnId;

    use super::{ADDR_TYPE_IDS, AddrTypeId};

    #[test]
    fn column_order_matches_named_series() {
        assert_eq!(AddrTypeId::ALL, ADDR_TYPE_IDS);

        let series = AddrTypeId::series(|column, _| column);
        assert!(series.values().copied().eq(ADDR_TYPE_IDS));
    }

    #[test]
    fn row_order_matches_column_indexes() {
        let row = AddrTypeId::from_fn(|column| column.index());

        for column in ADDR_TYPE_IDS {
            assert_eq!(*column.get(&row), column.index());
        }
    }
}
