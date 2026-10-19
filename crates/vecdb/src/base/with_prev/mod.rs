pub mod default;

/// Tracks current and previous values for rollback support.
#[derive(Debug, Clone)]
pub struct WithPrev<T> {
    pub(crate) current: T,
    pub(crate) previous: T,
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

    /// Copies current into previous.
    #[inline]
    pub(crate) fn save(&mut self)
    where
        T: Clone,
    {
        self.previous.clone_from(&self.current);
    }

    /// Copies previous into current.
    #[inline]
    pub(crate) fn restore(&mut self)
    where
        T: Clone,
    {
        self.current.clone_from(&self.previous);
    }

    #[inline]
    pub(crate) fn clear(&mut self)
    where
        T: Default,
    {
        self.current = T::default();
        self.previous = T::default();
    }
}
