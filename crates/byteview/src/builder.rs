use std::ops::{Deref, DerefMut};

use crate::ByteView;

/// A builder for a [`ByteView`] that allows mutation before freezing it.
pub struct Builder(ByteView);

impl Builder {
    /// Creates a new builder.
    #[must_use]
    pub(super) const fn new(inner: ByteView) -> Self {
        Self(inner)
    }

    /// Converts the builder into a [`ByteView`], making it immutable.
    #[must_use]
    pub fn freeze(self) -> ByteView {
        self.0
    }
}

impl Deref for Builder {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Builder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_mut_slice()
    }
}
