use std::mem;

pub mod default;

/// Tracks current and previous values for rollback support.
#[derive(Debug, Clone)]
pub struct WithPrev<T> {
    current: T,
    previous: T,
}

impl<T> WithPrev<T> {
    pub(crate) fn new(value: T) -> Self
    where
        T: Clone,
    {
        Self {
            current: value.clone(),
            previous: value,
        }
    }

    #[inline(always)]
    pub(crate) fn current(&self) -> &T {
        &self.current
    }

    #[inline]
    pub(crate) fn current_mut(&mut self) -> &mut T {
        &mut self.current
    }

    #[inline(always)]
    pub(crate) fn previous(&self) -> &T {
        &self.previous
    }

    #[inline]
    pub(crate) fn previous_mut(&mut self) -> &mut T {
        &mut self.previous
    }

    /// Copies current into previous.
    #[inline]
    pub(crate) fn save(&mut self)
    where
        T: Clone,
    {
        self.previous.clone_from(&self.current);
    }

    #[inline]
    pub(crate) fn take_current(&mut self) -> T
    where
        T: Default,
    {
        mem::take(&mut self.current)
    }

    #[inline]
    pub(crate) fn clear(&mut self)
    where
        T: Default,
    {
        self.current = T::default();
        self.previous = T::default();
    }

    #[inline]
    pub(crate) fn clear_previous(&mut self)
    where
        T: Default,
    {
        self.previous = T::default();
    }
}
