use brk_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Flags<V> {
    /// Whether the transaction is heuristically classified as a CoinJoin
    /// candidate: at least five inputs and outputs, neither count five times
    /// the other, sufficiently repeated input/output values, no recognized
    /// address reuse, and no detected `OP_RETURN` or inscription.
    pub is_coinjoin: V,
    /// Whether the transaction has at least five times as many inputs as
    /// outputs.
    pub is_consolidation: V,
    /// Whether the transaction is non-coinbase and has at least five times as
    /// many outputs as inputs.
    pub is_batch_payout: V,
}
