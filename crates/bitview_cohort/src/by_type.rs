use std::{
    iter,
    ops::{Add, AddAssign},
};

use brk_types::OutputType;

use super::{CohortId, SpendableType, UnspendableType};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

pub(crate) const OP_RETURN: &str = "op_return";
#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByType<T> {
    #[cfg_attr(feature = "storage", traversable(flatten))]
    pub spendable: SpendableType<T>,
    #[cfg_attr(feature = "storage", traversable(flatten))]
    pub unspendable: UnspendableType<T>,
}

impl<T> ByType<T> {
    pub fn from_fn(mut create: impl FnMut(OutputType) -> T) -> Self {
        Self {
            spendable: SpendableType::from_fn(|kind| create(kind.output_type())),
            unspendable: UnspendableType {
                op_return: create(OutputType::OpReturn),
            },
        }
    }

    pub fn map_with_id<U>(&self, mut map: impl FnMut(CohortId, &T) -> U) -> ByType<U> {
        ByType {
            spendable: self.spendable.map_with_id(&mut map),
            unspendable: UnspendableType {
                op_return: map(
                    CohortId::Type(OutputType::OpReturn),
                    &self.unspendable.op_return,
                ),
            },
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(CohortId) -> Result<T, E>,
    {
        Ok(Self {
            spendable: SpendableType::try_new(&mut create)?,
            unspendable: UnspendableType {
                op_return: create(CohortId::Type(OutputType::OpReturn))?,
            },
        })
    }

    pub fn get(&self, output_type: OutputType) -> &T {
        match output_type {
            OutputType::OpReturn => &self.unspendable.op_return,
            kind => self.spendable.get(kind),
        }
    }

    pub fn get_mut(&mut self, output_type: OutputType) -> &mut T {
        match output_type {
            OutputType::OpReturn => &mut self.unspendable.op_return,
            kind => self.spendable.get_mut(kind),
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.spendable
            .iter_mut()
            .chain(iter::once(&mut self.unspendable.op_return))
    }

    pub fn iter_typed_mut(&mut self) -> impl Iterator<Item = (OutputType, &mut T)> {
        self.spendable.iter_typed_mut().chain(iter::once((
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
    fn typed_iteration_preserves_collection_order() {
        let mut types = ByType::from_fn(|kind| kind);
        let expected = [
            OutputType::P2PK65,
            OutputType::P2PK33,
            OutputType::P2PKH,
            OutputType::P2MS,
            OutputType::P2SH,
            OutputType::P2WPKH,
            OutputType::P2WSH,
            OutputType::P2TR,
            OutputType::P2A,
            OutputType::Unknown,
            OutputType::Empty,
            OutputType::OpReturn,
        ];
        assert_eq!(expected.len(), OutputType::COUNT);
        assert!(types.iter_mut().map(|kind| *kind).eq(expected));
        assert!(types.iter_typed_mut().map(|(kind, _)| kind).eq(expected));
        for (kind, value) in types.iter_typed_mut() {
            assert_eq!(kind, *value);
        }
        for kind in expected {
            assert_eq!(*types.get(kind), kind);
        }

        let mut values = ByType::from_fn(|kind| kind as usize);
        for (kind, value) in values.iter_typed_mut() {
            *value = kind as usize + 1;
        }
        for kind in expected {
            assert_eq!(*values.get(kind), kind as usize + 1);
            *values.get_mut(kind) += 1;
        }
        for (kind, value) in values.iter_typed_mut() {
            assert_eq!(*value, kind as usize + 2);
        }
    }
}
