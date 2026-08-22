use bitview_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct CpfpFlags<V> {
    /// Whether the transaction is a child-pays-for-parent (CPFP) parent: its
    /// Single Fee Linearization (SFL) effective fee rate is higher than its raw fee
    /// rate because same-block descendants raise the rate at which its SFL
    /// chunk is evaluated.
    pub is_cpfp_parent: V,
    /// Whether the transaction is a child-pays-for-parent (CPFP) child: its
    /// Single Fee Linearization (SFL) effective fee rate is lower than its raw fee
    /// rate because its fee raises the rate at which a same-block
    /// ancestor-closed SFL chunk is evaluated.
    pub is_cpfp_child: V,
}
