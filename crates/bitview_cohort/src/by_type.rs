use std::{
    iter,
    ops::{Add, AddAssign},
};

use brk_types::OutputType;

use super::{CohortId, SpendableType, SpendableTypeId, UnspendableType};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

pub(crate) const OP_RETURN: &str = "op_return";

/// An output type as analytics count it: a spendable type or OP_RETURN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTypeId {
    Spendable(SpendableTypeId),
    OpReturn,
}

impl OutputTypeId {
    /// Members of [`ByType`]: the spendable types, then OP_RETURN.
    pub const COUNT: usize = SpendableTypeId::ALL.len() + 1;

    pub fn all() -> impl Iterator<Item = Self> {
        SpendableTypeId::ALL
            .iter()
            .map(|&kind| Self::Spendable(kind))
            .chain(iter::once(Self::OpReturn))
    }

    #[inline]
    pub const fn from_output_type(output_type: OutputType) -> Self {
        match SpendableTypeId::from_output_type(output_type) {
            Some(kind) => Self::Spendable(kind),
            None => Self::OpReturn,
        }
    }

    /// Position among [`OutputTypeId::all`], for per-member counters.
    #[inline]
    pub const fn index(self) -> usize {
        match self {
            Self::Spendable(kind) => kind.index(),
            Self::OpReturn => SpendableTypeId::ALL.len(),
        }
    }

    /// The member key, the word per-entry series ids build on (`p2pk_output_count`).
    pub const fn key(self) -> &'static str {
        match self {
            Self::Spendable(kind) => kind.key(),
            Self::OpReturn => OP_RETURN,
        }
    }

    pub const fn cohort(self) -> CohortId {
        match self {
            Self::Spendable(kind) => kind.cohort(),
            Self::OpReturn => CohortId::OpReturn,
        }
    }

    pub fn select<T>(self, values: &ByType<T>) -> &T {
        match self {
            Self::Spendable(kind) => kind.select(&values.spendable),
            Self::OpReturn => &values.unspendable.op_return,
        }
    }
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByType<T> {
    #[cfg_attr(feature = "storage", traversable(flatten))]
    pub spendable: SpendableType<T>,
    #[cfg_attr(feature = "storage", traversable(flatten))]
    pub unspendable: UnspendableType<T>,
}

impl<T> ByType<T> {
    pub fn from_fn(mut create: impl FnMut(OutputTypeId) -> T) -> Self {
        Self {
            spendable: SpendableType::from_fn(|kind| create(OutputTypeId::Spendable(kind))),
            unspendable: UnspendableType {
                op_return: create(OutputTypeId::OpReturn),
            },
        }
    }

    pub fn try_from_fn<E>(mut create: impl FnMut(OutputTypeId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            spendable: SpendableType::try_from_fn(|kind| create(OutputTypeId::Spendable(kind)))?,
            unspendable: UnspendableType {
                op_return: create(OutputTypeId::OpReturn)?,
            },
        })
    }

    /// The member holding `output_type`; P2PK's 33- and 65-byte keys share one.
    pub fn get(&self, output_type: OutputType) -> &T {
        OutputTypeId::from_output_type(output_type).select(self)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.spendable
            .iter_mut()
            .chain(iter::once(&mut self.unspendable.op_return))
    }

    pub fn iter_typed_mut(&mut self) -> impl Iterator<Item = (OutputTypeId, &mut T)> {
        OutputTypeId::all().zip(self.iter_mut())
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
