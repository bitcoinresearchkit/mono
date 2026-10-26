#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct UnspendableType<T> {
    /// Uses outputs whose locking script begins with `OP_RETURN`.
    #[cfg_attr(feature = "storage", traversable(lead = "OP_RETURN outputs: data written on the chain, never spendable."))]
    pub op_return: T,
}
