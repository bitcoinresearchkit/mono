use std::fmt::Debug;

/// A vector value: implemented for every type meeting the bounds.
pub trait VecValue
where
    Self: Sized + Debug + Clone + Send + Sync + 'static,
{
}

impl<T> VecValue for T where T: Sized + Debug + Clone + Send + Sync + 'static {}
