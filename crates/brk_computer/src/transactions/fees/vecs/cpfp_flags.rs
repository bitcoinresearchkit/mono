use brk_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct CpfpFlags<V> {
    /// Whether the transaction's Single Fee Linearization effective fee rate
    /// is higher than its raw fee rate, indicating that same-block descendants
    /// raise the rate at which the transaction's SFL chunk is evaluated.
    pub is_cpfp_parent: V,
    /// Whether the transaction's Single Fee Linearization effective fee rate
    /// is lower than its raw fee rate, indicating that its fee raises the rate
    /// at which a same-block ancestor-closed SFL chunk is evaluated.
    pub is_cpfp_child: V,
}
