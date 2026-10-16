// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

//! A fork of <https://github.com/dodomorandi/double-ended-peekable>
//! to allow accessing the inner type
//!
//! Also changes the generics a bit so it plays well with `self_cell`.

/// An _extension trait_ to create [`DoubleEndedPeekable`].
///
/// This has a blanket implementation for all types that implement [`Iterator`].
pub trait DoubleEndedPeekableExt<T, I: Iterator<Item = T>> {
    /// Creates an iterator which works similarly to [`Peekable`], but also provides additional
    /// functions if the underlying type implements [`DoubleEndedIterator`].
    ///
    /// See [`DoubleEndedPeekable`] for more information.
    ///
    /// [`Peekable`]: core::iter::Peekable
    fn double_ended_peekable(self) -> DoubleEndedPeekable<T, I>;
}

impl<T, I> DoubleEndedPeekableExt<T, I> for I
where
    I: Iterator<Item = T>,
{
    #[inline]
    fn double_ended_peekable(self) -> DoubleEndedPeekable<T, I> {
        DoubleEndedPeekable {
            iter: self,
            front: None,
            back: None,
        }
    }
}

/// An advanced version of [`Peekable`] that works well with double-ended iterators.
///
/// This `struct` is created by the [`double_ended_peekable`] method on [`DoubleEndedPeekableExt`].
///
/// [`Peekable`]: core::iter::Peekable
/// [`double_ended_peekable`]: DoubleEndedPeekableExt::double_ended_peekable
#[expect(
    clippy::option_option,
    reason = "`None` until peeked; `Some(None)` caches the end of the iteration"
)]
pub struct DoubleEndedPeekable<T, I: Iterator<Item = T>> {
    iter: I,
    front: Option<Option<T>>,
    back: Option<Option<T>>,
}

impl<T, I> DoubleEndedPeekable<T, I>
where
    I: Iterator<Item = T>,
{
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.iter
    }

    /// Returns a reference to the `next()` value without advancing the iterator.
    ///
    /// See [`Peekable::peek`] for more information.
    ///
    /// [`Peekable::peek`]: core::iter::Peekable::peek
    #[inline]
    pub fn peek(&mut self) -> Option<&I::Item> {
        self.front
            .get_or_insert_with(|| self.iter.next())
            .as_ref()
            .or_else(|| self.back.as_ref().and_then(Option::as_ref))
    }

    /// Consumes and returns the next value of this iterator if a condition is true.
    ///
    /// See [`Peekable::next_if`] for more information.
    ///
    /// [`Peekable::next_if`]: core::iter::Peekable::next_if
    #[inline]
    pub fn next_if(&mut self, func: impl FnOnce(&I::Item) -> bool) -> Option<I::Item> {
        match self.next() {
            Some(item) if func(&item) => Some(item),
            other => {
                debug_assert!(self.front.is_none());
                self.front = Some(other);
                None
            }
        }
    }
}

impl<T, I> DoubleEndedPeekable<T, I>
where
    I: DoubleEndedIterator<Item = T>,
{
    /// Returns a reference to the `next_back()` value without advancing the _back_ of the iterator.
    ///
    /// Like [`next_back`], if there is a value, it is wrapped in a `Some(T)`.
    /// But if the iteration is over, `None` is returned.
    ///
    /// [`next_back`]: DoubleEndedIterator::next_back
    ///
    /// Because `peek_back()` returns a reference, and many iterators iterate over references,
    /// there can be a possibly confusing situation where the return value is a double reference.
    #[inline]
    pub fn peek_back(&mut self) -> Option<&I::Item> {
        self.back
            .get_or_insert_with(|| self.iter.next_back())
            .as_ref()
            .or_else(|| self.front.as_ref().and_then(Option::as_ref))
    }
}

impl<T, I> Iterator for DoubleEndedPeekable<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.front
            .take()
            .unwrap_or_else(|| self.iter.next())
            .or_else(|| self.back.take().flatten())
    }
}

impl<T, I> DoubleEndedIterator for DoubleEndedPeekable<T, I>
where
    I: DoubleEndedIterator<Item = T>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.back
            .take()
            .unwrap_or_else(|| self.iter.next_back())
            .or_else(|| self.front.take().flatten())
    }
}
