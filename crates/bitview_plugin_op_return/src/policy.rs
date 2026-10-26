use bitview_traversable::Traversable;

#[derive(Clone, Copy, Default, Traversable)]
pub struct Policy<T> {
    /// Restricted to transactions with exactly one OP_RETURN output containing
    /// at most 82 post-OP_RETURN bytes: within the pre-v30 relay policy's size and
    /// count limits.
    #[traversable(lead = "Within the old relay limits: transactions with one OP_RETURN output, of no more than the size Bitcoin Core allowed before version 30.")]
    pub pre_v30_standard: T,
    /// Restricted to transactions with an oversized OP_RETURN output or more
    /// than one OP_RETURN output: beyond the pre-v30 relay policy's size or count
    /// limits.
    #[traversable(lead = "Beyond the old relay limits: transactions with a larger OP_RETURN output than Bitcoin Core allowed before version 30, or several.")]
    pub pre_v30_nonstandard: T,
    /// Restricted to transactions with at least one OP_RETURN output containing
    /// more than 82 post-OP_RETURN bytes. Output counts and data bytes count only
    /// those oversized outputs.
    #[traversable(lead = "Large OP_RETURN outputs: over the old relay limit's size.")]
    pub oversized: T,
    /// Restricted to transactions containing more than one OP_RETURN output.
    #[traversable(lead = "Transactions with more than one OP_RETURN output.")]
    pub multiple: T,
}

impl<T> Policy<T> {
    pub fn try_new<E>(mut create: impl FnMut(&'static str) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            pre_v30_standard: create("pre_v30_standard")?,
            pre_v30_nonstandard: create("pre_v30_nonstandard")?,
            oversized: create("oversized")?,
            multiple: create("multiple")?,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.pre_v30_standard,
            &mut self.pre_v30_nonstandard,
            &mut self.oversized,
            &mut self.multiple,
        ]
        .into_iter()
    }
}

impl<T> IntoIterator for Policy<T> {
    type Item = T;
    type IntoIter = std::array::IntoIter<T, 4>;

    /// The values in field order, as [`Self::iter_mut`].
    fn into_iter(self) -> Self::IntoIter {
        [
            self.pre_v30_standard,
            self.pre_v30_nonstandard,
            self.oversized,
            self.multiple,
        ]
        .into_iter()
    }
}
