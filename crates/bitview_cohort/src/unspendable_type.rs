use std::ops::{Add, AddAssign};

use bitview_traversable::Traversable;

#[derive(Default, Clone, Debug, Traversable)]
pub struct UnspendableType<T> {
    /// Uses outputs whose locking script begins with `OP_RETURN`.
    pub op_return: T,
}

impl<T> UnspendableType<T> {
    pub fn as_vec(&self) -> [&T; 1] {
        [&self.op_return]
    }
}

impl<T> Add for UnspendableType<T>
where
    T: Add<Output = T>,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            op_return: self.op_return + rhs.op_return,
        }
    }
}

impl<T> AddAssign for UnspendableType<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.op_return += rhs.op_return;
    }
}
