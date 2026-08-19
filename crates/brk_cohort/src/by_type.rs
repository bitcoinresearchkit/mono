use std::ops::{Add, AddAssign};

use bitview_traversable::Traversable;
use brk_types::OutputType;
use rayon::prelude::*;
use vecdb::{ColumnId, VecValue, Version};

use super::{Filter, SpendableType, UnspendableType};

pub const OP_RETURN: &str = "op_return";
pub const OUTPUT_TYPE_COUNT: usize = OutputType::COUNT;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum OutputTypeId {
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
    OpReturn,
}

pub const OUTPUT_TYPE_IDS: [OutputTypeId; OUTPUT_TYPE_COUNT] = [
    OutputTypeId::P2PK65,
    OutputTypeId::P2PK33,
    OutputTypeId::P2PKH,
    OutputTypeId::P2MS,
    OutputTypeId::P2SH,
    OutputTypeId::P2WPKH,
    OutputTypeId::P2WSH,
    OutputTypeId::P2TR,
    OutputTypeId::P2A,
    OutputTypeId::Unknown,
    OutputTypeId::Empty,
    OutputTypeId::OpReturn,
];

impl OutputTypeId {
    pub const fn from_output_type(value: OutputType) -> Self {
        match value {
            OutputType::P2PK65 => Self::P2PK65,
            OutputType::P2PK33 => Self::P2PK33,
            OutputType::P2PKH => Self::P2PKH,
            OutputType::P2MS => Self::P2MS,
            OutputType::P2SH => Self::P2SH,
            OutputType::P2WPKH => Self::P2WPKH,
            OutputType::P2WSH => Self::P2WSH,
            OutputType::P2TR => Self::P2TR,
            OutputType::P2A => Self::P2A,
            OutputType::Unknown => Self::Unknown,
            OutputType::Empty => Self::Empty,
            OutputType::OpReturn => Self::OpReturn,
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
            Self::OpReturn => OutputType::OpReturn,
        }
    }
}

impl ColumnId for OutputTypeId {
    type Row<T>
        = [T; OUTPUT_TYPE_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &OUTPUT_TYPE_IDS;

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
        std::array::from_fn(|index| f(OUTPUT_TYPE_IDS[index]))
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

#[derive(Default, Clone, Debug, Traversable)]
pub struct ByType<T> {
    #[traversable(flatten)]
    pub spendable: SpendableType<T>,
    #[traversable(flatten)]
    pub unspendable: UnspendableType<T>,
}

impl<T> ByType<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        Self {
            spendable: SpendableType::new(&mut create),
            unspendable: UnspendableType {
                op_return: create(Filter::Type(OutputType::OpReturn), OP_RETURN),
            },
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        Ok(Self {
            spendable: SpendableType::try_new(&mut create)?,
            unspendable: UnspendableType {
                op_return: create(Filter::Type(OutputType::OpReturn), OP_RETURN)?,
            },
        })
    }

    pub fn get(&self, output_type: OutputType) -> &T {
        match output_type {
            OutputType::P2PK65 => &self.spendable.p2pk65,
            OutputType::P2PK33 => &self.spendable.p2pk33,
            OutputType::P2PKH => &self.spendable.p2pkh,
            OutputType::P2MS => &self.spendable.p2ms,
            OutputType::P2SH => &self.spendable.p2sh,
            OutputType::P2WPKH => &self.spendable.p2wpkh,
            OutputType::P2WSH => &self.spendable.p2wsh,
            OutputType::P2TR => &self.spendable.p2tr,
            OutputType::P2A => &self.spendable.p2a,
            OutputType::Empty => &self.spendable.empty,
            OutputType::Unknown => &self.spendable.unknown,
            OutputType::OpReturn => &self.unspendable.op_return,
        }
    }

    pub fn get_mut(&mut self, output_type: OutputType) -> &mut T {
        match output_type {
            OutputType::P2PK65 => &mut self.spendable.p2pk65,
            OutputType::P2PK33 => &mut self.spendable.p2pk33,
            OutputType::P2PKH => &mut self.spendable.p2pkh,
            OutputType::P2MS => &mut self.spendable.p2ms,
            OutputType::P2SH => &mut self.spendable.p2sh,
            OutputType::P2WPKH => &mut self.spendable.p2wpkh,
            OutputType::P2WSH => &mut self.spendable.p2wsh,
            OutputType::P2TR => &mut self.spendable.p2tr,
            OutputType::P2A => &mut self.spendable.p2a,
            OutputType::Unknown => &mut self.spendable.unknown,
            OutputType::Empty => &mut self.spendable.empty,
            OutputType::OpReturn => &mut self.unspendable.op_return,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.spendable
            .iter()
            .chain(std::iter::once(&self.unspendable.op_return))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.spendable
            .iter_mut()
            .chain(std::iter::once(&mut self.unspendable.op_return))
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        let Self {
            spendable,
            unspendable,
        } = self;
        spendable
            .par_iter_mut()
            .chain([&mut unspendable.op_return].into_par_iter())
    }

    pub fn iter_typed(&self) -> impl Iterator<Item = (OutputType, &T)> {
        self.spendable.iter_typed().chain(std::iter::once((
            OutputType::OpReturn,
            &self.unspendable.op_return,
        )))
    }

    pub fn iter_typed_mut(&mut self) -> impl Iterator<Item = (OutputType, &mut T)> {
        self.spendable.iter_typed_mut().chain(std::iter::once((
            OutputType::OpReturn,
            &mut self.unspendable.op_return,
        )))
    }
}

impl<T> Add for ByType<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            spendable: self.spendable + rhs.spendable,
            unspendable: self.unspendable + rhs.unspendable,
        }
    }
}

impl<T> AddAssign for ByType<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.spendable += rhs.spendable;
        self.unspendable += rhs.unspendable;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_ids_match_by_type_order() {
        let by_type = ByType::new(|filter, _| {
            let Filter::Type(output_type) = filter else {
                unreachable!()
            };
            output_type
        });
        let output_types: Vec<_> = by_type.iter().copied().collect();
        let column_output_types: Vec<_> = OutputTypeId::ALL
            .iter()
            .map(|column| column.output_type())
            .collect();

        assert_eq!(column_output_types, output_types);

        let row = OutputTypeId::from_fn(|column| column.index());
        for column in OutputTypeId::ALL {
            assert_eq!(*column.get(&row), column.index());
        }
    }
}
